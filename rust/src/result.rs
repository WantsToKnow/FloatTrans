use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::DataExchange::*;
use windows::Win32::System::Memory::*;
use windows::Win32::Graphics::Gdi::{
    CreateFontIndirectW, HFONT, LOGFONTW, FW_NORMAL, ANTIALIASED_QUALITY,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const RESULT_CLASS: PCWSTR = w!("FloatTransResult");
pub const WM_APP_CONTINUOUS_TOGGLE: u32 = 0x8002;
pub const WM_APP_RESULT_CLOSED: u32 = 0x8003;

const IDC_EN_TEXT: isize = 1001;
const IDC_ZH_TEXT: isize = 1002;
const IDC_COPY_EN: isize = 1003;
const IDC_COPY_ZH: isize = 1004;
const IDC_OK: isize = 1005;
const IDC_EN_LABEL: isize = 1006;
const IDC_ZH_LABEL: isize = 1007;
const IDC_CONTINUOUS: isize = 1008;

const WIN_W: i32 = 560;
const WIN_H: i32 = 440;

struct ResultData {
    en: String,
    zh: String,
    ball_hwnd: HWND,
    cont_on: bool,
}

pub extern "system" fn result_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                // 全局 alpha 半透明(200/255), 控件可正常点击
                let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 200, LWA_ALPHA);

                let cs = &*(lp.0 as *const CREATESTRUCTW);
                let data = &*(cs.lpCreateParams as *const ResultData);
                let hinst = cs.hInstance;

                macro_rules! ctl {
                    ($class:expr, $text:expr, $style:expr, $x:expr, $y:expr, $w:expr, $h:expr, $id:expr) => {
                        let _ = CreateWindowExW(
                            WINDOW_EX_STYLE::default(),
                            $class, $text, $style,
                            $x, $y, $w, $h, hwnd, $id, hinst, None,
                        );
                    };
                }

                ctl!(w!("STATIC"), w!("英文 (OCR):"), WS_CHILD | WS_VISIBLE, 12, 8, 520, 18,
                    HMENU(IDC_EN_LABEL as *mut core::ffi::c_void));
                ctl!(w!("EDIT"), w!(""),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | ES_MULTILINE as u32 | ES_READONLY as u32
                        | ES_AUTOVSCROLL as u32 | WS_VSCROLL.0),
                    12, 28, 520, 100, HMENU(IDC_EN_TEXT as *mut core::ffi::c_void));
                ctl!(w!("BUTTON"), w!("复制英文"),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | BS_PUSHBUTTON as u32),
                    424, 134, 108, 26, HMENU(IDC_COPY_EN as *mut core::ffi::c_void));

                ctl!(w!("STATIC"), w!("中文 (翻译):"), WS_CHILD | WS_VISIBLE, 12, 170, 520, 18,
                    HMENU(IDC_ZH_LABEL as *mut core::ffi::c_void));
                ctl!(w!("EDIT"), w!(""),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | ES_MULTILINE as u32 | ES_READONLY as u32
                        | ES_AUTOVSCROLL as u32 | WS_VSCROLL.0),
                    12, 190, 520, 100, HMENU(IDC_ZH_TEXT as *mut core::ffi::c_void));
                ctl!(w!("BUTTON"), w!("复制中文"),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | BS_PUSHBUTTON as u32),
                    424, 296, 108, 26, HMENU(IDC_COPY_ZH as *mut core::ffi::c_void));

                ctl!(w!("BUTTON"), w!("确定"),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | BS_PUSHBUTTON as u32),
                    436, 340, 96, 30, HMENU(IDC_OK as *mut core::ffi::c_void));
                ctl!(w!("BUTTON"), w!("持续翻译: 开"),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | BS_PUSHBUTTON as u32),
                    12, 340, 120, 30, HMENU(IDC_CONTINUOUS as *mut core::ffi::c_void));

                // 初始文本
                let en_h = HSTRING::from(&data.en);
                let _ = SetWindowTextW(GetDlgItem(hwnd, IDC_EN_TEXT as i32).unwrap_or_default(), &en_h);
                let zh_h = HSTRING::from(&data.zh);
                let _ = SetWindowTextW(GetDlgItem(hwnd, IDC_ZH_TEXT as i32).unwrap_or_default(), &zh_h);

                let boxed = Box::new(ResultData {
                    en: data.en.clone(), zh: data.zh.clone(), ball_hwnd: data.ball_hwnd, cont_on: true,
                });
                let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(boxed) as isize);

                let hfont = create_ui_font();
                if !hfont.is_invalid() {
                    for id in [IDC_EN_TEXT, IDC_ZH_TEXT, IDC_COPY_EN, IDC_COPY_ZH, IDC_OK, IDC_CONTINUOUS, IDC_EN_LABEL, IDC_ZH_LABEL] {
                        let _ = SendMessageW(
                            GetDlgItem(hwnd, id as i32).unwrap_or_default(),
                            WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(1),
                        );
                    }
                }

                // 默认开启持续翻译
                let _ = PostMessageW(data.ball_hwnd, WM_APP_CONTINUOUS_TOGGLE, WPARAM(1), LPARAM(hwnd.0 as isize));

                LRESULT(0)
            }

            WM_COMMAND => {
                let id = (wp.0 & 0xFFFF) as isize;
                match id {
                    IDC_COPY_EN => copy_edit_text(hwnd, IDC_EN_TEXT as i32),
                    IDC_COPY_ZH => copy_edit_text(hwnd, IDC_ZH_TEXT as i32),
                    IDC_CONTINUOUS => {
                        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ResultData;
                        if !ptr.is_null() {
                            (*ptr).cont_on = !(*ptr).cont_on;
                            let on = (*ptr).cont_on;
                            let txt = if on { w!("持续翻译: 开") } else { w!("持续翻译: 关") };
                            let _ = SetWindowTextW(GetDlgItem(hwnd, IDC_CONTINUOUS as i32).unwrap_or_default(), txt);
                            let _ = PostMessageW((*ptr).ball_hwnd, WM_APP_CONTINUOUS_TOGGLE,
                                WPARAM(if on { 1 } else { 0 }), LPARAM(hwnd.0 as isize));
                        }
                    }
                    IDC_OK => {
                        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ResultData;
                        if !ptr.is_null() {
                            let _ = PostMessageW((*ptr).ball_hwnd, WM_APP_RESULT_CLOSED, WPARAM(0), LPARAM(0));
                        }
                        let _ = DestroyWindow(hwnd);
                    }
                    _ => {}
                }
                LRESULT(0)
            }

            WM_CLOSE => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ResultData;
                if !ptr.is_null() {
                    let _ = PostMessageW((*ptr).ball_hwnd, WM_APP_RESULT_CLOSED, WPARAM(0), LPARAM(0));
                }
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }

            WM_DESTROY => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ResultData;
                if !ptr.is_null() { drop(Box::from_raw(ptr)); }
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}

pub fn update_text(hwnd: HWND, en: &str, zh: &str) {
    unsafe {
        let en_h = HSTRING::from(en);
        let _ = SetWindowTextW(GetDlgItem(hwnd, IDC_EN_TEXT as i32).unwrap_or_default(), &en_h);
        let zh_h = HSTRING::from(zh);
        let _ = SetWindowTextW(GetDlgItem(hwnd, IDC_ZH_TEXT as i32).unwrap_or_default(), &zh_h);
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ResultData;
        if !ptr.is_null() { (*ptr).en = en.to_string(); (*ptr).zh = zh.to_string(); }
    }
}

fn copy_edit_text(hwnd: HWND, edit_id: i32) {
    unsafe {
        let edit = match GetDlgItem(hwnd, edit_id) { Ok(h) => h, Err(_) => return };
        let len = GetWindowTextLengthW(edit);
        if len == 0 { return; }
        let cap = (len + 1) as usize;
        let mut buf: Vec<u16> = vec![0; cap];
        if GetWindowTextW(edit, &mut buf) == 0 { return; }
        let bytes = cap * 2;
        let hmem = match GlobalAlloc(GMEM_MOVEABLE, bytes) { Ok(h) => h, Err(_) => return };
        let ptr = GlobalLock(hmem);
        if !ptr.is_null() {
            std::ptr::copy_nonoverlapping(buf.as_ptr(), ptr as *mut u16, cap);
            let _ = GlobalUnlock(hmem);
        }
        if OpenClipboard(hwnd).is_ok() {
            let _ = EmptyClipboard();
            let _ = SetClipboardData(CF_UNICODETEXT.0 as u32, HANDLE(hmem.0));
            let _ = CloseClipboard();
        }
    }
}

fn create_ui_font() -> HFONT {
    let mut lf = LOGFONTW::default();
    lf.lfHeight = -13;
    lf.lfWeight = FW_NORMAL.0 as i32;
    lf.lfQuality = ANTIALIASED_QUALITY;
    let name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    for (i, &c) in name.iter().enumerate().take(32) { lf.lfFaceName[i] = c; }
    unsafe { CreateFontIndirectW(&lf) }
}

pub fn show(hinst: HINSTANCE, owner: HWND, en: &str, zh: &str) -> Result<HWND> {
    let data = ResultData { en: en.to_string(), zh: zh.to_string(), ball_hwnd: owner, cont_on: true };
    let (cx, cy);
    unsafe {
        cx = (GetSystemMetrics(SM_CXSCREEN) - WIN_W) / 2;
        cy = (GetSystemMetrics(SM_CYSCREEN) - WIN_H) / 2;
    }
    unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_DLGMODALFRAME,
            RESULT_CLASS,
            w!("FloatTrans 翻译结果"),
            WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
            cx.max(0), cy.max(0), WIN_W, WIN_H,
            None, None, hinst,
            Some(&data as *const ResultData as *const core::ffi::c_void),
        )
    }
}
