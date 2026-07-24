use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const BALL_CLASS: PCWSTR = w!("FloatTransBall");
pub const BALL_SIZE: i32 = 54;

pub extern "system" fn ball_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let dc = BeginPaint(hwnd, &mut ps);
                if !dc.is_invalid() {
                    // 2x 离屏位图 + HALFTONE 缩放 = 抗锯齿圆
                    let scale = 2;
                    let big = BALL_SIZE * scale;
                    let mem_dc = CreateCompatibleDC(dc);
                    let bmp = CreateCompatibleBitmap(dc, big, big);
                    let old_bmp = SelectObject(mem_dc, bmp);

                    // 填充圆
                    let brush = CreateSolidBrush(COLORREF(0x00FFB82F)); // BGR #2FB8FF
                    let oldb = SelectObject(mem_dc, brush);
                    let _ = Ellipse(mem_dc, 0, 0, big, big);
                    SelectObject(mem_dc, oldb);
                    let _ = DeleteObject(brush);

                    // 2x → 1x 高质量缩放 (HALFTONE 模式带抗锯齿)
                    SetStretchBltMode(dc, HALFTONE);
                    let _ = StretchBlt(
                        dc, 0, 0, BALL_SIZE, BALL_SIZE,
                        mem_dc, 0, 0, big, big, SRCCOPY,
                    );

                    SelectObject(mem_dc, old_bmp);
                    let _ = DeleteObject(bmp);
                    let _ = DeleteDC(mem_dc);
                    let _ = EndPaint(hwnd, &ps);
                }
                LRESULT(0)
            }
            WM_TIMER => {
                match wp.0 {
                    id if id == crate::state::TIMER_ID as usize => {
                        crate::state::on_hold_tick();
                    }
                    id if id == crate::state::CONTINUOUS_TIMER_ID as usize => {
                        crate::state::on_continuous_tick();
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                match crate::tray::on_command(wp) {
                    Some(crate::tray::TrayAction::Exit) => {
                        PostQuitMessage(0);
                        return LRESULT(0);
                    }
                    Some(crate::tray::TrayAction::OpenConfig) => {
                        let (hinst, ball_hwnd) = crate::state::lock(|s| (s.hinst, s.ball_hwnd));
                        let _ = crate::config_win::show(hinst, ball_hwnd);
                        return LRESULT(0);
                    }
                    None => {}
                }
                DefWindowProcW(hwnd, msg, wp, lp)
            }
            _ => {
                if msg == crate::state::WM_APP_RESULT {
                    crate::state::on_result();
                    LRESULT(0)
                } else if msg == crate::tray::WM_APP_TRAY {
                    crate::tray::on_tray(hwnd, lp);
                    LRESULT(0)
                } else if msg == crate::result::WM_APP_CONTINUOUS_TOGGLE {
                    let on = wp.0 != 0;
                    crate::state::toggle_continuous(on, lp.0);
                    LRESULT(0)
                } else if msg == crate::result::WM_APP_RESULT_CLOSED {
                    // 结果窗关闭 → 清理选框
                    let was_cont = crate::state::CONTINUOUS.load(std::sync::atomic::Ordering::Relaxed);
                    if was_cont {
                        crate::state::toggle_continuous(false, 0);
                    } else {
                        crate::state::lock(|s| s.selection_done = false);
                        crate::overlay::exit_continuous();
                    }
                    LRESULT(0)
                } else {
                    DefWindowProcW(hwnd, msg, wp, lp)
                }
            }
        }
    }
}

pub fn create(hinst: HINSTANCE, x: i32, y: i32) -> Result<HWND> {
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TOOLWINDOW,
            BALL_CLASS,
            w!("FloatTrans"),
            WS_POPUP,
            x,
            y,
            BALL_SIZE,
            BALL_SIZE,
            None,
            None,
            hinst,
            None,
        )?
    };
    unsafe {
        let rgn = CreateRoundRectRgn(0, 0, BALL_SIZE, BALL_SIZE, BALL_SIZE, BALL_SIZE);
        if !rgn.is_invalid() {
            let _ = SetWindowRgn(hwnd, rgn, BOOL(1));
        }
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 160, LWA_ALPHA);
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
    Ok(hwnd)
}

pub fn set_center(hwnd: HWND, phys_x: i32, phys_y: i32) {
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            phys_x - BALL_SIZE / 2,
            phys_y - BALL_SIZE / 2,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

pub fn get_rect(hwnd: HWND) -> RECT {
    let mut r = RECT::default();
    unsafe {
        let _ = GetWindowRect(hwnd, &mut r);
    }
    r
}

pub fn hide(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

pub fn show(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
}
