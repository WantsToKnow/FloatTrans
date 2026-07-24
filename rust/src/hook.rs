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
    unsafe {
        let _ = GetCursorPos(&mut p);
    }
    p
}

extern "system" fn hook_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code >= 0 {
        let info: &MSLLHOOKSTRUCT = unsafe { &*(lp.0 as *const MSLLHOOKSTRUCT) };
        let p = info.pt;
        match wp.0 as u32 {
            WM_LBUTTONDOWN => crate::state::on_down(p),
            WM_MOUSEMOVE => crate::state::on_move(p),
            WM_LBUTTONUP => crate::state::on_up(p),
            _ => {}
        }
    }
    unsafe { CallNextHookEx(None, code, wp, lp) }
}
