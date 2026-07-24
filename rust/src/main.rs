mod ball;
mod capture;
mod config;
mod config_win;
mod hook;
mod ocr;
mod overlay;
mod result;
mod state;
mod tesseract;
mod translate;
mod tray;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() -> Result<()> {
    // 确保 DPI 感知生效,否则 Windows 会位图缩放 → 模糊
    unsafe {
        let ctx = DPI_AWARENESS_CONTEXT((-4isize) as *mut core::ffi::c_void); // PER_MONITOR_AWARE_V2
        if SetProcessDpiAwarenessContext(ctx).is_err() {
            // PMv2 失败(可能已被清单/注册表设置),回退到系统 DPI 感知
            let _ = SetProcessDPIAware();
        }
    }

    unsafe {
        let hinst: HINSTANCE = GetModuleHandleW(None)?.into();

        // 注册悬浮球窗口类
        let wc_ball = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(ball::ball_proc),
            hInstance: hinst,
            lpszClassName: ball::BALL_CLASS,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };
        let atom = RegisterClassExW(&wc_ball);
        debug_assert!(atom != 0, "RegisterClassExW ball failed");

        // 注册框选 overlay 窗口类
        let wc_ov = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(overlay::overlay_proc),
            hInstance: hinst,
            lpszClassName: overlay::OVERLAY_CLASS,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };
        let atom2 = RegisterClassExW(&wc_ov);
        debug_assert!(atom2 != 0, "RegisterClassExW overlay failed");

        // 注册结果窗口类
        let wc_res = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(result::result_proc),
            hInstance: hinst,
            lpszClassName: result::RESULT_CLASS,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH((COLOR_WINDOW.0 as isize + 1) as *mut core::ffi::c_void),
            ..Default::default()
        };
        let atom3 = RegisterClassExW(&wc_res);
        debug_assert!(atom3 != 0, "RegisterClassExW result failed");

        // 注册配置窗口类
        let wc_cfg = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(config_win::config_win_proc),
            hInstance: hinst,
            lpszClassName: config_win::CONFIG_WIN_CLASS,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH((COLOR_BTNFACE.0 as isize + 1) as *mut core::ffi::c_void),
            ..Default::default()
        };
        let atom4 = RegisterClassExW(&wc_cfg);
        debug_assert!(atom4 != 0, "RegisterClassExW config failed");

        // 创建悬浮球(初始位置 200,200)
        let ball_hwnd = ball::create(hinst, 200, 200)?;

        // 创建 overlay(隐藏,layered + 黑色 colorkey 透明)
        let overlay_hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW,
            overlay::OVERLAY_CLASS,
            w!(""),
            WS_POPUP,
            0,
            0,
            100,
            100,
            None,
            None,
            hinst,
            None,
        )?;
        let _ = SetLayeredWindowAttributes(overlay_hwnd, COLORREF(0), 0, LWA_COLORKEY);
        let _ = ShowWindow(overlay_hwnd, SW_HIDE);
        overlay::set_hwnd(overlay_hwnd);

        // 初始化全局状态
        let cfg = config::Config::load();
        let ocr_engine = ocr::Ocr::new();
        state::init(state::AppState {
            ball_hwnd,
            overlay_hwnd,
            hinst,
            state: state::State::Idle,
            down_point: POINT::default(),
            drag_offset: POINT::default(),
            select_start: POINT::default(),
            select_end: POINT::default(),
            cfg,
            ocr: ocr_engine,
            result_en: String::new(),
            result_zh: String::new(),
            result_hwnd: HWND::default(),
            last_ocr: String::new(),
            selection_done: false,
        });

        // 添加系统托盘图标
        tray::add(ball_hwnd);

        // 装全局鼠标钩子
        hook::install()?;

        // 消息循环
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // 退出前清理
        tray::remove(ball_hwnd);
    }
    Ok(())
}
