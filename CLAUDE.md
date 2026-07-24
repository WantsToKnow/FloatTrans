# CLAUDE.md — FloatTrans

Windows 桌面悬浮球截屏翻译工具。长按悬浮球 → 十字标框选 → 截图 → 本地 OCR(中英双语) → 百度翻译 API(自动检测→中文)。

## 项目架构

```
两层实现，C# WPF 为早期原型（已弃用），Rust 为当前主力版本。

┌─────────────────────────────────────────────────┐
│  交互层                                          │
│  悬浮球 (ball)  ──  框选蒙层 (overlay)            │
│       │                   │                      │
│   ┌───┴───────────────┴───┐                      │
│   │  状态机 (state)        │  全局鼠标钩子 (hook)  │
│   │  Idle → Pressing       │  WH_MOUSE_LL         │
│   │  → Dragging/Selecting  │                      │
│   └───┬───────────────────┘                      │
│       │ 松开 → spawn 线程                          │
│   ┌───┴──────────────┐                           │
│   │  capture(BitBlt) │  截图 BGRA                 │
│   │  ocr(WinRT双引擎) │  英文/中文 OCR               │
│   │  translate(ureq)  │  百度翻译 from=auto→zh      │
│   └───┬──────────────┘                           │
│       │ PostMessage → 结果窗 / 托盘 / 配置窗        │
└─────────────────────────────────────────────────┘
```

## 技术栈

| 层 | Rust 版 (当前) | C# 版 (已弃用) |
|---|---|---|
| 语言 | Rust 1.97 msvc | C# 12 / .NET 8 WPF |
| Win32 | windows-rs 0.58 | P/Invoke |
| OCR | Windows.Media.Ocr (WinRT) | Windows.Media.Ocr (WinRT) |
| HTTP | ureq 2 + md-5 | HttpClient + MD5 |
| 序列化 | serde + serde_json | System.Text.Json |
| 二进制 | 1.4MB (zip 763KB) | 179MB 自包含 / ~1MB 框架依赖 |

### 依赖 (Cargo.toml)
- `windows` 0.58: Win32 (20+ features) + WinRT (Media_Ocr, Graphics_Imaging, Storage_Streams, Globalization)
- `ureq` 2 (tls), `md-5` 0.10, `serde` 1 (derive), `serde_json` 1
- Release: `opt-level=z`, `lto=true`, `codegen-units=1`, `strip=true`, `panic=abort`

## 目录结构

```
FloatTrans/
├── CLAUDE.md                     # ← 本文件
├── README.md                     # 用户文档（C# 版，需更新）
├── FloatTrans-0.1.0-win-x64.zip  # 打包好的 release
│
├── rust/                         # ★ 当前主力：Rust 重写
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # 入口：DPI→注册窗口类→创建球→托盘→钩子→消息循环
│       ├── ball.rs               # 悬浮球：layered 椭圆窗口 (54×54)
│       ├── overlay.rs            # 框选蒙层：layered colorkey 全屏透明画布
│       ├── state.rs              # 状态机 + 全局 OnceLock<Mutex<AppState>>
│       ├── hook.rs               # SetWindowsHookExW(WH_MOUSE_LL)
│       ├── capture.rs            # BitBlt + GetDIBits → BGRA
│       ├── ocr.rs                # WinRT OcrEngine 中英双引擎 + 预处理
│       ├── translate.rs          # 百度翻译 POST + MD5 签名
│       ├── config.rs             # serde 配置读写 %AppData%\FloatTrans\config.json
│       ├── config_win.rs         # 配置窗口：AppId/Secret/长按毫秒 + 测试翻译
│       ├── result.rs             # 结果窗口：只读编辑框 + 复制英文/中文按钮
│       └── tray.rs               # Shell_NotifyIconW 托盘图标 + 右键菜单
│
├── FloatTrans/                   # C# WPF 旧版（功能完整，作参考）
│   ├── FloatTrans.csproj
│   ├── App.xaml/.cs              # 启动 + NotifyIcon 托盘
│   ├── MainWindow.xaml/.cs       # 悬浮球窗口
│   ├── OverlayWindow.xaml/.cs    # 框选画布
│   ├── ResultWindow.xaml/.cs     # 翻译结果窗
│   ├── ConfigWindow.xaml/.cs     # 配置窗口
│   └── Services/
│       ├── GlobalMouseHook.cs    # WH_MOUSE_LL
│       ├── SelectionController.cs# 状态机
│       ├── OcrService.cs         # WinRT OCR (单英文引擎)
│       ├── TranslateService.cs   # 百度翻译 POST
│       ├── ScreenCapture.cs      # CopyFromScreen
│       ├── MonitorHelper.cs      # 多屏 DPI
│       ├── DpiHelper.cs          # 坐标换算
│       └── AppConfig.cs          # JSON 配置
│
├── FloatTrans.sln
├── build.bat                     # C# 构建脚本
└── release/                      # 打包输出目录
```

## 常用命令

### Rust 版（当前）

```bash
# 注意：cargo 不在 PATH，需用完整路径；Bash 下需 dangerouslyDisableSandbox

# Debug 编译
/c/Users/link/.cargo/bin/cargo.exe build --manifest-path d:/aaa_file/study/FloatTrans/rust/Cargo.toml

# Release 编译（生产用）
/c/Users/link/.cargo/bin/cargo.exe build --release --manifest-path d:/aaa_file/study/FloatTrans/rust/Cargo.toml

# Release exe 位置：d:/aaa_file/study/FloatTrans/rust/target/release/floattrans.exe

# 打包 zip
powershell -Command "Compress-Archive -Path d:/aaa_file/study/FloatTrans/rust/target/release/floattrans.exe -DestinationPath d:/aaa_file/study/FloatTrans/FloatTrans-0.1.0-win-x64.zip -Force"
```

### C# 版（已弃用，仅参考）

```bash
dotnet build FloatTrans/FloatTrans.csproj -c Release
dotnet publish FloatTrans/FloatTrans.csproj -c Release -o publish
```

## 环境依赖

| 依赖 | 说明 |
|---|---|
| Rust 1.97 msvc | `/c/Users/link/.cargo/bin/cargo.exe`，清华镜像已配 |
| VS 2022 BuildTools | C++ workload（windows-rs 编译需要 MSVC linker） |
| Windows 10/11 | WinRT OCR 引擎 |
| OCR 语言包 | 英文 `en-US` + 中文 `zh-Hans-CN`（设置→语言→添加语言→勾选 OCR） |
| 百度翻译 API | 免费标准版 QPS=1，config.json 已配 AppId |

## 代码风格与约定

### Rust
- **模块边界**：每个窗口/功能一个文件，通过 `mod` 声明 + `pub` 导出
- **错误处理**：Win32 返回值用 `let _ = ...` 吞掉（不 panic，静默降级），关键路径用 `?` + `Result<()>`
- **unsafe**：所有 Win32 调用在 `unsafe {}` 块内，函数级不加 `unsafe`
- **状态管理**：`OnceLock<Mutex<AppState>>` + `lock(|s| ...)` 闭包模式，不暴露锁细节
- **跨线程**：HWND 传 `as usize`，线程内 `HWND(raw as *mut c_void)` 重建
- **命名**：模块 snake_case，常量 SCREAMING_SNAKE_CASE，函数 snake_case，struct CamelCase
- **注释**：中文注释解释"为什么"而非"做什么"，API 坑点需标注 windows-rs 版本
- **窗口过程**：`extern "system" fn` 返回 `LRESULT`，用 `match msg` 分发

### C#（旧版）
- WPF 窗口 + 代码后置
- 服务类在 `Services/` 下，每个类单一职责
- 异步方法以 `Async` 结尾
- 全局 using 别名解决 WPF/WinForms 类型冲突

## windows-rs 0.58 关键 API 差异

- `Language::CreateLanguage(&HSTRING::from(tag))` — 不是 `new()`
- `BitmapEncoder::SetPixelData(&[u8])` — 无 `AsBuffer`，直接传切片
- `CreatePen`/`CreateRoundRectRgn` 返回句柄非 `Result`，用 `is_invalid()` 检查
- `SetWindowRgn` 第二参直接 `HRGN`，非 `Option<HRGN>`
- `DPI_AWARENESS_CONTEXT` 是 `*mut c_void`，`PER_MONITOR_AWARE_V2 = -4`
- `WS_EX_NONE` 不存在，用 `WINDOW_EX_STYLE::default()`
- `ES_*`/`BS_*` 是 `i32`，用 `as u32`；`WS_*` 是 newtype，用 `.0` 取原始值
- `CF_UNICODETEXT` 在 `Win32_System_Ole`
- `MessageBoxW(None, ...)` — 无 owner 传 `None`；有 owner 直接传 `HWND`
- WinRT async: `op.get()` 阻塞，`.Text()` 取 `HString`
- 子控件 ID: `HMENU(id as *mut c_void)` 传给 `CreateWindowExW` 的 hmenu

## 状态机

```
Idle ──(点在球上 + 左键按下)──→ Pressing
  ↑                              │
  ├──(短按松开)──────────────────┘
  │                               │
  │                          SetTimer(hold_ms)
  │                               │
  ├──(拖动 > 5px)──← Dragging ←──┤ (移动 > 5px)
  │                               │
  │                           WM_TIMER 触发
  │                               │
  └──(松开左键)──← Selecting ←────┘
                    │
                    └→ spawn thread: hide ball+overlay → sleep 100ms → capture → OCR → translate → PostMessage(WM_APP_RESULT)
```

## 当前状态

**所有计划功能已实现，无待办。**

- ✅ 悬浮球 (layered 椭圆, 54×54)
- ✅ 长按框选 (0.5s, 可配置)
- ✅ 双屏 DPI (PerMonitorV2)
- ✅ 中英双语 OCR (zh-Hans-CN + en 双引擎, 含中文优先用 zh)
- ✅ 百度翻译 (from=auto, to=zh)
- ✅ 托盘图标 (Shell_NotifyIconW + 右键菜单"配置"/"退出")
- ✅ 配置窗口 (AppId/Secret/长按毫秒 + 测试翻译按钮)
- ✅ 结果窗口 (非模态, 英文/中文编辑框可选中复制 + 复制按钮)
- ✅ Release 1.4MB (zip 763KB)
- ✅ 启动无需 .NET 运行时

## 未来可扩展

- Pin 功能：框选后固定区域，每 500ms 自动 OCR 刷新
- 自定义悬浮球图标 / 贴图
- 多翻译引擎支持 (DeepL, Google 等)
- 热键触发框选（不依赖长按悬浮球）
- 更细粒度的 Windows 消息日志 / 诊断模式
