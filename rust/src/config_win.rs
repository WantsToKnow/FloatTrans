use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::{
    CreateFontIndirectW, FONT_QUALITY, HFONT, LOGFONTW, FW_NORMAL, CLEARTYPE_NATURAL_QUALITY,
};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::{config::Config, state, translate};

pub const CONFIG_WIN_CLASS: PCWSTR = w!("FloatTransConfig");

const IDC_APPID: i32 = 2001;
const IDC_SECRET: i32 = 2002;
const IDC_HOLD_MS: i32 = 2003;
const IDC_TEST: i32 = 2004;
const IDC_SAVE: i32 = 2005;
const IDC_CANCEL: i32 = 2006;

const WIN_W: i32 = 420;
const WIN_H: i32 = 260;

/// 编辑框 → Rust String
fn get_text(hwnd: HWND, id: i32) -> String {
    unsafe {
        let edit = match GetDlgItem(hwnd, id) {
            Ok(h) => h,
            Err(_) => return String::new(),
        };
        let len = GetWindowTextLengthW(edit) as usize;
        if len == 0 {
            return String::new();
        }
        let mut buf: Vec<u16> = vec![0; len + 1];
        GetWindowTextW(edit, &mut buf);
        String::from_utf16_lossy(&buf[..len])
    }
}

/// 从编辑框读取配置
fn read_config(hwnd: HWND) -> Config {
    Config {
        baidu_app_id: get_text(hwnd, IDC_APPID),
        baidu_secret: get_text(hwnd, IDC_SECRET),
        hold_ms: get_text(hwnd, IDC_HOLD_MS).parse().unwrap_or(500),
    }
}

/// 用字符串填充编辑框
fn set_text(hwnd: HWND, id: i32, s: &str) {
    let h = HSTRING::from(s);
    let edit = unsafe { GetDlgItem(hwnd, id).unwrap_or_default() };
    unsafe {
        let _ = SetWindowTextW(edit, &h);
    }
}

pub extern "system" fn config_win_proc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let cs = &*(lp.0 as *const CREATESTRUCTW);
                let hinst = cs.hInstance;

                // 辅助宏
                macro_rules! ctl {
                    ($class:expr, $text:expr, $style:expr, $x:expr, $y:expr, $w:expr, $h:expr, $id:expr) => {
                        let _ = CreateWindowExW(
                            WINDOW_EX_STYLE::default(),
                            $class,
                            $text,
                            $style,
                            $x,
                            $y,
                            $w,
                            $h,
                            hwnd,
                            $id,
                            hinst,
                            None,
                        );
                    };
                }

                // 百度 APP ID
                ctl!(
                    w!("STATIC"), w!("百度 APP ID:"),
                    WS_CHILD | WS_VISIBLE,
                    12, 12, 100, 18,
                    HMENU::default()
                );
                ctl!(
                    w!("EDIT"), w!(""),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | WS_BORDER.0 | ES_AUTOHSCROLL as u32),
                    12, 34, 394, 22,
                    HMENU(IDC_APPID as *mut core::ffi::c_void)
                );

                // 百度 Secret
                ctl!(
                    w!("STATIC"), w!("百度 Secret:"),
                    WS_CHILD | WS_VISIBLE,
                    12, 64, 120, 18,
                    HMENU::default()
                );
                ctl!(
                    w!("EDIT"), w!(""),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | WS_BORDER.0 | ES_AUTOHSCROLL as u32),
                    12, 86, 394, 22,
                    HMENU(IDC_SECRET as *mut core::ffi::c_void)
                );

                // 长按毫秒
                ctl!(
                    w!("STATIC"), w!("长按毫秒:"),
                    WS_CHILD | WS_VISIBLE,
                    12, 116, 80, 18,
                    HMENU::default()
                );
                ctl!(
                    w!("EDIT"), w!(""),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | WS_BORDER.0 | ES_NUMBER as u32),
                    12, 138, 120, 22,
                    HMENU(IDC_HOLD_MS as *mut core::ffi::c_void)
                );

                // 测试翻译按钮
                ctl!(
                    w!("BUTTON"), w!("测试翻译"),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | BS_PUSHBUTTON as u32),
                    150, 136, 100, 26,
                    HMENU(IDC_TEST as *mut core::ffi::c_void)
                );

                // 保存按钮
                ctl!(
                    w!("BUTTON"), w!("保存"),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | BS_PUSHBUTTON as u32),
                    240, 180, 80, 26,
                    HMENU(IDC_SAVE as *mut core::ffi::c_void)
                );

                // 取消按钮
                ctl!(
                    w!("BUTTON"), w!("取消"),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | BS_PUSHBUTTON as u32),
                    328, 180, 80, 26,
                    HMENU(IDC_CANCEL as *mut core::ffi::c_void)
                );

                // 从全局状态读取当前配置并填充
                state::lock(|s| {
                    set_text(hwnd, IDC_APPID, &s.cfg.baidu_app_id);
                    set_text(hwnd, IDC_SECRET, &s.cfg.baidu_secret);
                    set_text(hwnd, IDC_HOLD_MS, &s.cfg.hold_ms.to_string());
                });

                // 设置 Segoe UI + ClearType 字体
                let hfont = create_ui_font();
                if !hfont.is_invalid() {
                    for id in [IDC_APPID, IDC_SECRET, IDC_HOLD_MS] {
                        let _ = SendMessageW(
                            GetDlgItem(hwnd, id).unwrap_or_default(),
                            WM_SETFONT,
                            WPARAM(hfont.0 as usize),
                            LPARAM(1),
                        );
                    }
                }

                LRESULT(0)
            }

            WM_COMMAND => {
                let id = (wp.0 & 0xFFFF) as i32;
                match id {
                    IDC_TEST => {
                        let cfg = read_config(hwnd);
                        let result = translate::translate_to_zh("hello", &cfg);
                        let ok = translate::is_success(&result);
                        let title = if ok {
                            w!("测试成功")
                        } else {
                            w!("测试失败")
                        };
                        let h = HSTRING::from(&result);
                        let _ = MessageBoxW(hwnd, &h, title, MB_OK | MB_ICONINFORMATION);
                    }
                    IDC_SAVE => {
                        let cfg = read_config(hwnd);
                        cfg.save();
                        state::update_config(cfg);
                        let _ = DestroyWindow(hwnd);
                    }
                    IDC_CANCEL => {
                        let _ = DestroyWindow(hwnd);
                    }
                    _ => {}
                }
                LRESULT(0)
            }

            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}

/// 创建 Segoe UI 字体 (ClearType, 10pt)
fn create_ui_font() -> HFONT {
    let mut lf = LOGFONTW::default();
    lf.lfHeight = -13;
    lf.lfWeight = FW_NORMAL.0 as i32;
    lf.lfQuality = FONT_QUALITY(CLEARTYPE_NATURAL_QUALITY as u8);
    let name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    for (i, &c) in name.iter().enumerate().take(32) {
        lf.lfFaceName[i] = c;
    }
    unsafe { CreateFontIndirectW(&lf) }
}

/// 显示配置窗口(模态对话框形式)
pub fn show(hinst: HINSTANCE, owner: HWND) -> Result<HWND> {
    let cx;
    let cy;
    unsafe {
        cx = (GetSystemMetrics(SM_CXSCREEN) - WIN_W) / 2;
        cy = (GetSystemMetrics(SM_CYSCREEN) - WIN_H) / 2;
    }

    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME,
            CONFIG_WIN_CLASS,
            w!("FloatTrans 配置"),
            WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
            cx.max(0),
            cy.max(0),
            WIN_W,
            WIN_H,
            owner,
            None,
            hinst,
            None,
        )?;

        Ok(hwnd)
    }
}
