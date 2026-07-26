use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn install() -> Result<()> {
    let hmod = unsafe { GetModuleHandleW(None)? };
    let _h = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), hmod, 0) }?;
    Ok(())
}

pub fn get_cursor_pos() -> POINT {
    let mut p = POINT::default();
    unsafe { let _ = GetCursorPos(&mut p); }
    p
}

extern "system" fn hook_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code >= 0 {
        let info: &MSLLHOOKSTRUCT = unsafe { &*(lp.0 as *const MSLLHOOKSTRUCT) };
        let p = info.pt;
        let msg = wp.0 as u32;

        // 持续模式下: 红框内的点击事件吃掉, 不穿透到下层
        if crate::state::CONTINUOUS.load(std::sync::atomic::Ordering::Relaxed) {
            if (msg == WM_LBUTTONDOWN || msg == WM_LBUTTONUP)
                && should_consume(p)
            {
                if msg == WM_LBUTTONDOWN { crate::state::on_down(p); }
                if msg == WM_LBUTTONUP   { crate::state::on_up(p); }
                return LRESULT(1); // 吃掉! 不传给下层窗口
            }
        }

        match msg {
            WM_LBUTTONDOWN => crate::state::on_down(p),
            WM_MOUSEMOVE => crate::state::on_move(p),
            WM_LBUTTONUP => crate::state::on_up(p),
            _ => {}
        }
    }
    unsafe { CallNextHookEx(None, code, wp, lp) }
}

/// 判断鼠标是否在选框内(且不在结果窗上)
fn should_consume(p: POINT) -> bool {
    let in_rect = crate::overlay::point_in_selection(p);
    if !in_rect { return false; }
    // 如果点在结果窗上, 不消费(让结果窗可操作)
    let rh = crate::state::lock(|s| s.result_hwnd);
    if rh.0.is_null() { return true; }
    unsafe {
        let target = WindowFromPoint(p);
        target != rh && !IsChild(rh, target).as_bool()
    }
}
