# FloatTrans — Windows 桌面悬浮球截屏翻译

类似安卓 FV 悬浮球的轻量翻译工具。

- 鼠标停在悬浮球上,**左键长按 1.5s** → 出现十字标(进入框选模式)
- 左键不松,移动鼠标**拖出矩形框**
- 松开左键 → 自动截取矩形 → **本地 OCR 识别英文** → **联网翻译成中文** → 弹出结果窗口

| 环节 | 实现 |
|------|------|
| 图片 → 英文 | Windows.Media.Ocr(系统自带,本地离线,免费) |
| 英文 → 中文 | 百度翻译通用 API(免费标准版,QPS=1) |
| 平台 | C# WPF,.NET 8,单文件 exe,框架依赖 |

## 交互

1. 悬浮球默认在屏幕右下角;**短按拖动**可移动其位置
2. 鼠标停在球上,**左键长按 1.5s** → 出现红色十字标
3. 左键不松,移动鼠标画出蓝色矩形框
4. **松开左键** → 截图 → OCR → 翻译 → 弹出结果窗口
5. 结果窗口可一键复制英文 / 中文

## 前置要求

### 1. .NET 8 Desktop Runtime(运行时)
发布为框架依赖单文件,需安装 .NET 8 桌面运行时:
https://dotnet.microsoft.com/download/dotnet/8.0 → 选 **.NET Desktop Runtime**

### 2. 英文 OCR 语言包(本地 OCR)
**设置 → 时间和语言 → 语言 → 添加语言 → English (United States) → 勾选“OCR”**

或用 PowerShell(管理员):
```powershell
$Capability = Get-WindowsCapability -Online | Where-Object { $_.Name -Like 'Language.OCR*en-US*' }
$Capability | Add-WindowsCapability -Online
```

### 3. 百度翻译 API(免费)
1. 注册并登录:https://fanyi-api.baidu.com/
2. 控制台 → 开通「通用翻译 API」(标准版免费,QPS=1)
3. 获取 **APP ID** 和 **密钥**
4. 首次运行本程序会自动生成配置文件:`%AppData%\FloatTrans\config.json`,填入 `BaiduAppId` / `BaiduSecret` 后重启程序

## 构建

需要 **.NET 8 SDK**:
```bash
dotnet build FloatTrans/FloatTrans.csproj -c Release
```

发布为单文件:
```bash
dotnet publish FloatTrans/FloatTrans.csproj -c Release -o publish
```
产物:`publish\FloatTrans.exe`(框架依赖,需目标机装 .NET 8 Desktop Runtime)。

或双击 `build.bat`。

## 配置文件

路径:`%AppData%\FloatTrans\config.json`
```json
{
  "BaiduAppId": "你的appid",
  "BaiduSecret": "你的密钥",
  "HoldMilliseconds": 1500,
  "BallSize": 54,
  "BallOpacity": 0.55
}
```

## 目录结构
```
FloatTrans/
├── FloatTrans.sln
├── build.bat
├── README.md
└── FloatTrans/
    ├── FloatTrans.csproj
    ├── app.manifest          (DPI: System awareness)
    ├── App.xaml / .cs        (启动:配置→DPI→钩子→悬浮球)
    ├── MainWindow.xaml/.cs   (悬浮球:无边框置顶透明穿透)
    ├── OverlayWindow.xaml/.cs(全屏框选画布:十字标+矩形)
    ├── ResultWindow.xaml/.cs (结果窗口:英文/中文,可复制)
    └── Services/
        ├── GlobalMouseHook.cs   (WH_MOUSE_LL 全局低级鼠标钩子)
        ├── DpiHelper.cs         (物理↔逻辑坐标换算)
        ├── ScreenCapture.cs     (CopyFromScreen 截图)
        ├── OcrService.cs        (Windows.Media.Ocr)
        ├── TranslateService.cs  (百度翻译 + MD5 签名 + QPS 节流)
        ├── AppConfig.cs         (JSON 配置)
        └── SelectionController.cs(状态机:长按→框选→截图→OCR→翻译)
```

## 已知限制
- **DPI**:以系统 DPI(System awareness)工作;多显示器混合缩放下框选可能略有偏差
- **截图**:`CopyFromScreen` 无法截取受 DRM 保护或部分硬件加速内容(可能黑屏)
- **QPS**:百度标准版 QPS=1,程序已内置 1.1s 间隔节流
- **鼠标钩子**:低级钩子需应用有消息循环;回调长时间阻塞(>300ms)会被系统摘钩,故 OCR/翻译均异步
