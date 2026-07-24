use std::sync::atomic::{AtomicBool, Ordering};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const OVERLAY_CLASS: PCWSTR = w!("FloatTransOverlay");

fn is_continuous() -> bool { crate::state::CONTINUOUS.load(Ordering::Relaxed) }
static DRAGGING: AtomicBool = AtomicBool::new(false);
static RESIZING: AtomicBool = AtomicBool::new(false);
// (mouse_down_point, select_start_at_down, select_end_at_down)
static DRAG_BASE: std::sync::Mutex<(POINT, POINT, POINT)> = std::sync::Mutex::new((
    POINT { x: 0, y: 0 },
    POINT { x: 0, y: 0 },
    POINT { x: 0, y: 0 },
));
static OVERLAY_HWND: std::sync::OnceLock<usize> = std::sync::OnceLock::new();

pub fn set_hwnd(h: HWND) { let _ = OVERLAY_HWND.set(h.0 as usize); }
fn get_hwnd() -> HWND { HWND(OVERLAY_HWND.get().copied().unwrap_or(0) as *mut core::ffi::c_void) }

pub extern "system" fn overlay_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let dc = BeginPaint(hwnd, &mut ps);
                if dc.is_invalid() { return LRESULT(0); }

                let black = GetStockObject(BLACK_BRUSH);
                let _ = FillRect(dc, &ps.rcPaint, HBRUSH(black.0));

                if let Some((s, e)) = crate::state::get_selection() {
                    let mut wr = RECT::default();
                    let _ = GetWindowRect(hwnd, &mut wr);
                    let (ox, oy) = (wr.left, wr.top);
                    let sx = s.x - ox; let sy = s.y - oy;
                    let ex = e.x - ox; let ey = e.y - oy;
                    let x = sx.min(ex); let y = sy.min(ey);
                    let w = (sx - ex).abs(); let h = (sy - ey).abs();

                    let null_brush = GetStockObject(NULL_BRUSH);
                    let pen = CreatePen(PS_SOLID, 1, COLORREF(0x000000FF));
                    let oldb = SelectObject(dc, HBRUSH(null_brush.0));
                    let oldp = SelectObject(dc, pen);
                    let _ = Rectangle(dc, x, y, x + w, y + h);
                    SelectObject(dc, oldb); SelectObject(dc, oldp);
                    let _ = DeleteObject(pen);

                    let cpen = CreatePen(PS_SOLID, 1, COLORREF(0x000000FF));
                    if !cpen.is_invalid() {
                        let oldcp = SelectObject(dc, cpen);
                        let _ = MoveToEx(dc, ex - 12, ey, None);
                        let _ = LineTo(dc, ex + 12, ey);
                        let _ = MoveToEx(dc, ex, ey - 12, None);
                        let _ = LineTo(dc, ex, ey + 12);
                        SelectObject(dc, oldcp);
                        let _ = DeleteObject(cpen);
                    }
                }
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}

fn hit_test(hwnd: HWND, px: i32, py: i32) -> bool {
    if let Some((s, e)) = crate::state::get_selection() {
        let mut wr = RECT::default();
        unsafe { let _ = GetWindowRect(hwnd, &mut wr); }
        let (ox, oy) = (wr.left, wr.top);
        let sx = s.x - ox; let sy = s.y - oy;
        let ex = e.x - ox; let ey = e.y - oy;
        let rx = sx.min(ex); let ry = sy.min(ey);
        let rw = (sx - ex).abs(); let rh = (sy - ey).abs();
        const M: i32 = 6;
        px >= rx - M && px <= rx + rw + M && py >= ry - M && py <= ry + rh + M
    } else {
        false
    }
}

/// 检查是否点在右下角(红十字位置)附近 → 触发 resize
fn corner_test(hwnd: HWND, px: i32, py: i32) -> bool {
    if let Some((_, e)) = crate::state::get_selection() {
        let mut wr = RECT::default();
        unsafe { let _ = GetWindowRect(hwnd, &mut wr); }
        let (ox, oy) = (wr.left, wr.top);
        let ex = e.x - ox;
        let ey = e.y - oy;
        const R: i32 = 15;
        (px - ex).abs() <= R && (py - ey).abs() <= R
    } else {
        false
    }
}

fn monitor_rect(p: POINT) -> RECT {
    let hm = unsafe { MonitorFromPoint(p, MONITOR_DEFAULTTONEAREST) };
    let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    unsafe { let _ = GetMonitorInfoW(hm, &mut mi); }
    mi.rcMonitor
}

pub fn begin(hwnd: HWND, phys: POINT) {
    let rc = monitor_rect(phys);
    unsafe {
        let _ = SetWindowPos(hwnd, HWND_TOPMOST, rc.left, rc.top,
            rc.right - rc.left, rc.bottom - rc.top, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        let _ = RedrawWindow(hwnd, None, None, RDW_INVALIDATE | RDW_ERASE);
    }
}

pub fn invalidate(hwnd: HWND) {
    unsafe {
        let mut rc = RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);
        let _ = InvalidateRect(hwnd, Some(&rc), true);
    }
}

pub fn enter_continuous() {}
pub fn exit_continuous() {
    DRAGGING.store(false, Ordering::Relaxed);
    let hwnd = get_hwnd();
    unsafe { let _ = ShowWindow(hwnd, SW_HIDE); }
}

/// 钩子: 按下 → 判断框内拖拽/右下角拉伸
pub fn on_down(p: POINT) {
    if !is_continuous() { return; }
    let hwnd = get_hwnd();
    let mut wr = RECT::default();
    unsafe { let _ = GetWindowRect(hwnd, &mut wr); }
    let px = p.x - wr.left;
    let py = p.y - wr.top;
    if corner_test(hwnd, px, py) {
        RESIZING.store(true, Ordering::Relaxed);
        let (s0, e0) = crate::state::lock(|s| (s.select_start, s.select_end));
        *DRAG_BASE.lock().unwrap() = (p, s0, e0);
    } else if hit_test(hwnd, px, py) {
        DRAGGING.store(true, Ordering::Relaxed);
        let (s0, e0) = crate::state::lock(|s| (s.select_start, s.select_end));
        *DRAG_BASE.lock().unwrap() = (p, s0, e0);
    }
}

/// 钩子: 拖拽移动/拉伸选框
pub fn on_move(p: POINT) {
    if RESIZING.load(Ordering::Relaxed) {
        let (base_pt, _base_s, base_e) = *DRAG_BASE.lock().unwrap();
        let dx = p.x - base_pt.x;
        let dy = p.y - base_pt.y;
        crate::state::resize_selection(base_e, dx, dy);
        invalidate(get_hwnd());
    } else if DRAGGING.load(Ordering::Relaxed) {
        let (base_pt, base_s, base_e) = *DRAG_BASE.lock().unwrap();
        let dx = p.x - base_pt.x;
        let dy = p.y - base_pt.y;
        crate::state::move_selection(base_s, base_e, dx, dy);
        invalidate(get_hwnd());
    }
}

/// 钩子: 松开
pub fn on_up(_p: POINT) {
    DRAGGING.store(false, Ordering::Relaxed);
    RESIZING.store(false, Ordering::Relaxed);
}
