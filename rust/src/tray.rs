use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const WM_APP_TRAY: u32 = 0x8001;
const IDM_EXIT: usize = 1;
const IDM_CONFIG: usize = 2;

/// 托盘菜单动作
pub enum TrayAction {
    Exit,
    OpenConfig,
}

/// 添加系统托盘图标
pub fn add(hwnd: HWND) {
    let mut tip: [u16; 128] = [0; 128];
    let name: Vec<u16> = "FloatTrans\0".encode_utf16().collect();
    let len = name.len().min(127);
    tip[..len].copy_from_slice(&name[..len]);

    let icon = unsafe { LoadIconW(None, IDI_APPLICATION).unwrap_or_default() };

    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_APP_TRAY,
        hIcon: icon,
        szTip: tip,
        ..Default::default()
    };
    unsafe {
        let _ = Shell_NotifyIconW(NIM_ADD, &nid);
    }
}

/// 移除系统托盘图标
pub fn remove(hwnd: HWND) {
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        ..Default::default()
    };
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}

/// 处理托盘回调消息(通常是右键点击)
pub fn on_tray(hwnd: HWND, lparam: LPARAM) {
    let event = (lparam.0 as u32) & 0xFFFF;
    if event == WM_RBUTTONUP {
        unsafe {
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);

            let menu = match CreatePopupMenu() {
                Ok(m) => m,
                Err(_) => return,
            };

            let _ = AppendMenuW(menu, MF_STRING, IDM_CONFIG, w!("配置(&C)"));
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
            let _ = AppendMenuW(menu, MF_STRING, IDM_EXIT, w!("退出(&X)"));
            // SetForegroundWindow 确保菜单在点击其他地方时正确关闭
            let _ = SetForegroundWindow(hwnd);
            let _ = TrackPopupMenu(
                menu,
                TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                pt.x,
                pt.y,
                0,
                hwnd,
                None,
            );
            let _ = DestroyMenu(menu);
        }
    }
}

/// 处理菜单点击,返回 None 表示非托盘菜单项
pub fn on_command(wparam: WPARAM) -> Option<TrayAction> {
    match wparam.0 {
        IDM_EXIT => Some(TrayAction::Exit),
        IDM_CONFIG => Some(TrayAction::OpenConfig),
        _ => None,
    }
}
