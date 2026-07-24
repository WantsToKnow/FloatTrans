use std::sync::{atomic::AtomicBool, atomic::Ordering, Mutex, OnceLock};
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::{ball, capture, config, hook, ocr, overlay, result, translate};

pub const TIMER_ID: usize = 1001;
pub const CONTINUOUS_TIMER_ID: usize = 1002;
pub const WM_APP_RESULT: u32 = 0x8000;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    Pressing,
    Dragging,
    Selecting,
}

pub struct AppState {
    pub ball_hwnd: HWND,
    pub overlay_hwnd: HWND,
    pub hinst: HINSTANCE,
    pub state: State,
    pub down_point: POINT,
    pub drag_offset: POINT,
    pub select_start: POINT,
    pub select_end: POINT,
    pub cfg: config::Config,
    pub ocr: ocr::Ocr,
    pub result_en: String,
    pub result_zh: String,
    pub result_hwnd: HWND,
    pub last_ocr: String,
    pub selection_done: bool, // 初始框选完成, overlay 应持续显示红框
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();
/// 持续模式标志 — AtomicBool, 钩子回调无锁读取
pub static CONTINUOUS: AtomicBool = AtomicBool::new(false);

pub fn init(s: AppState) {
    let _ = STATE.set(Mutex::new(s));
}

pub fn lock<R>(f: impl FnOnce(&mut AppState) -> R) -> R {
    f(&mut *STATE.get().unwrap().lock().unwrap())
}

pub fn update_config(cfg: config::Config) {
    lock(|s| s.cfg = cfg);
}

fn on_ball(p: POINT, s: &AppState) -> bool {
    let r = ball::get_rect(s.ball_hwnd);
    p.x >= r.left && p.x <= r.right && p.y >= r.top && p.y <= r.bottom
}

pub fn on_down(p: POINT) {
    if CONTINUOUS.load(Ordering::Relaxed) {
        overlay::on_down(p);
        return;
    }
    lock(|s| {
        if s.state != State::Idle || !on_ball(p, s) { return; }
        s.down_point = p;
        s.state = State::Pressing;
        unsafe { let _ = SetTimer(s.ball_hwnd, TIMER_ID, s.cfg.hold_ms as u32, None); }
    });
}

pub fn on_move(p: POINT) {
    if CONTINUOUS.load(Ordering::Relaxed) {
        overlay::on_move(p);
        return;
    }
    lock(|s| match s.state {
        State::Pressing => {
            let dx = p.x - s.down_point.x;
            let dy = p.y - s.down_point.y;
            if dx * dx + dy * dy > 25 {
                unsafe { let _ = KillTimer(s.ball_hwnd, TIMER_ID); }
                let r = ball::get_rect(s.ball_hwnd);
                s.drag_offset = POINT {
                    x: (r.left + r.right) / 2 - s.down_point.x,
                    y: (r.top + r.bottom) / 2 - s.down_point.y,
                };
                s.state = State::Dragging;
                ball::set_center(s.ball_hwnd, p.x + s.drag_offset.x, p.y + s.drag_offset.y);
            }
        }
        State::Dragging => {
            ball::set_center(s.ball_hwnd, p.x + s.drag_offset.x, p.y + s.drag_offset.y)
        }
        State::Selecting => {
            s.select_end = p;
            overlay::invalidate(s.overlay_hwnd);
        }
        _ => {}
    });
}

pub fn on_up(p: POINT) {
    if CONTINUOUS.load(Ordering::Relaxed) {
        overlay::on_up(p);
        return;
    }
    let action = lock(|s| match s.state {
        State::Pressing => {
            unsafe { let _ = KillTimer(s.ball_hwnd, TIMER_ID); }
            s.state = State::Idle;
            None
        }
        State::Dragging => {
            s.state = State::Idle;
            None
        }
        State::Selecting => {
            s.state = State::Idle;
            Some((s.select_start, p))
        }
        _ => None,
    });
    if let Some((start, end)) = action {
        let ball_raw = lock(|s| {
            ball::hide(s.ball_hwnd);
            s.ball_hwnd.0 as usize
        });
        std::thread::spawn(move || process(start, end, ball_raw));
    }
}

pub fn on_hold_tick() {
    lock(|s| {
        if s.state == State::Pressing {
            unsafe { let _ = KillTimer(s.ball_hwnd, TIMER_ID); }
            s.state = State::Selecting;
            s.selection_done = false; // 新选框覆盖旧的
            let cur = hook::get_cursor_pos();
            s.select_start = cur;
            s.select_end = cur;
            overlay::begin(s.overlay_hwnd, cur);
        }
    });
}

pub fn on_result() {
    let (en, zh, ball_hwnd, hinst) = lock(|s| {
        (s.result_en.clone(), s.result_zh.clone(), s.ball_hwnd, s.hinst)
    });
    ball::show(ball_hwnd);
    let rh = result::show(hinst, ball_hwnd, &en, &zh).unwrap_or_default();
    // 确保结果窗在 overlay 之上, 否则 checkbox 点击被吞
    unsafe { let _ = SetWindowPos(rh, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE); }
    lock(|s| { s.result_hwnd = rh; s.selection_done = true; });
}

pub fn get_selection() -> Option<(POINT, POINT)> {
    lock(|s| {
        if s.state == State::Selecting || CONTINUOUS.load(Ordering::Relaxed) || s.selection_done {
            Some((s.select_start, s.select_end))
        } else {
            None
        }
    })
}

/// 清空选框(仅在开始新框选或退出持续模式时调用)
pub fn clear_selection() {
    lock(|s| {
        s.selection_done = false;
        s.select_start = POINT::default();
        s.select_end = POINT::default();
    });
}

pub fn move_selection(base_start: POINT, base_end: POINT, dx: i32, dy: i32) {
    lock(|s| {
        s.select_start = POINT { x: base_start.x + dx, y: base_start.y + dy };
        s.select_end = POINT { x: base_end.x + dx, y: base_end.y + dy };
    });
}

pub fn toggle_continuous(on: bool, result_hwnd_raw: isize) {
    lock(|s| {
        if on {
            s.result_hwnd = HWND(result_hwnd_raw as *mut core::ffi::c_void);
            unsafe { let _ = SetTimer(s.ball_hwnd, CONTINUOUS_TIMER_ID, 1000, None); }
        } else {
            s.result_hwnd = HWND::default();
            unsafe { let _ = KillTimer(s.ball_hwnd, CONTINUOUS_TIMER_ID); }
            s.last_ocr.clear();
            s.selection_done = false;
        }
    });
    CONTINUOUS.store(on, Ordering::Relaxed);
    if on { overlay::enter_continuous(); }
    else { overlay::exit_continuous(); }
}

/// 防止连续 tick 重叠
static TICK_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn on_continuous_tick() {
    if TICK_RUNNING.swap(true, Ordering::Relaxed) {
        return; // 上一轮还没跑完
    }
    // 从 state 克隆数据, 立即释放锁
    let job = lock(|s| {
        if !CONTINUOUS.load(Ordering::Relaxed) || s.result_hwnd.0.is_null() {
            return None;
        }
        let sx = s.select_start.x.min(s.select_end.x);
        let sy = s.select_start.y.min(s.select_end.y);
        let sw = (s.select_start.x - s.select_end.x).abs();
        let sh = (s.select_start.y - s.select_end.y).abs();
        if sw < 5 || sh < 5 { return None; }
        Some((sx, sy, sw, sh, s.result_hwnd, s.ocr.clone(), s.last_ocr.clone(), s.cfg.clone()))
    });

    if let Some((x, y, w, h, rh, o, last, cfg)) = job {
        let rh_raw = rh.0 as usize;
        // 后台线程跑慢操作, 不阻塞主线程(鼠标不卡)
        std::thread::spawn(move || {
            let rh = HWND(rh_raw as *mut core::ffi::c_void);
            let cap = match capture::capture(x, y, w, h) { Ok(c) => c, Err(_) => { TICK_RUNNING.store(false, Ordering::Relaxed); return; } };
            let en = match o.recognize(&cap.bgra, cap.width, cap.height) {
                Ok(t) => t,
                Err(_) => { TICK_RUNNING.store(false, Ordering::Relaxed); return; }
            };
            if en != last && !en.is_empty() {
                let zh = translate::translate_to_zh(&en, &cfg);
                result::update_text(rh, &en, &zh);
                lock(|s| s.last_ocr = en);
            }
            TICK_RUNNING.store(false, Ordering::Relaxed);
        });
    } else {
        TICK_RUNNING.store(false, Ordering::Relaxed);
    }
}

fn process(start: POINT, end: POINT, ball_raw: usize) {
    let ball_hwnd = HWND(ball_raw as *mut core::ffi::c_void);
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);
    let w = (start.x - end.x).abs();
    let h = (start.y - end.y).abs();
    if w < 5 || h < 5 {
        lock(|s| {
            s.result_en = String::new();
            s.result_zh = "(区域过小)".into();
        });
        unsafe { let _ = PostMessageW(ball_hwnd, WM_APP_RESULT, WPARAM(0), LPARAM(0)); }
        return;
    }
    std::thread::sleep(std::time::Duration::from_millis(100));

    let en = match capture::capture(x, y, w, h) {
        Ok(c) => lock(|s| s.ocr.recognize(&c.bgra, c.width, c.height).unwrap_or_default()),
        Err(e) => format!("[截图失败] {}", e),
    };
    let cfg = lock(|s| s.cfg.clone());
    let zh = translate::translate_to_zh(&en, &cfg);

    lock(|s| {
        s.result_en = en;
        s.result_zh = zh;
        s.last_ocr = s.result_en.clone();
    });
    unsafe { let _ = PostMessageW(ball_hwnd, WM_APP_RESULT, WPARAM(0), LPARAM(0)); }
}
