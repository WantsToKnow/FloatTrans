# FloatTrans — Windows 桌面悬浮球截屏翻译

类似安卓 FV 悬浮球，**长按悬浮球 → 框选 → 截图 OCR → 自动翻译**。

无需安装任何运行时，下载即用。

## 快速开始

1. `git clone` 本仓库（或直接下载 zip）
2. 打开 `rust/target/release/floattrans.exe`，双击运行
3. 屏幕出现蓝色悬浮球
4. 鼠标停在球上，**长按 0.5s** → 出现红色十字标
5. 拖动鼠标框选英文区域 → 松开 → 自动识别并翻译成中文
6. 持续翻译默认开启：红框保持、可拖拽/缩放、每秒自动刷新

## 前置要求

### 1. 百度翻译 API（唯一需要配置的）

1. 打开 [fanyi-api.baidu.com](https://fanyi-api.baidu.com/)，注册登录
2. 控制台 → 开通「通用翻译 API」（标准版**免费**，QPS=1）
3. 获取 **APP ID** 和**密钥**
4. 启动 FloatTrans 后，**右键右下角任务栏图标 → 配置**，填入凭据并保存
5. 点「测试翻译」确认配置正确

或在 `%AppData%\FloatTrans\config.json` 手动填写：

```json
{
  "BaiduAppId": "你的appid",
  "BaiduSecret": "你的密钥",
  "HoldMilliseconds": 500
}
```

### 2. OCR 引擎（已内置，无需额外安装）

仓库已包含 Tesseract OCR 引擎（`rust/target/release/tesseract/`），clone 即用。

## 操作说明

| 操作 | 效果 |
|------|------|
| 长按悬浮球 0.5s | 进入框选模式，出现红色十字标 |
| 拖动 | 画出红色矩形框选区 |
| 松开 | 截图 → OCR 识别英文 → 百度翻译 → 弹出结果 |
| 悬浮球短按拖动 | 移动悬浮球位置 |
| 结果窗「复制英文/中文」 | 复制到剪贴板 |
| 结果窗「持续翻译: 开」 | 红框保持，拖拽移动/右下角缩放，每秒自动翻译 |
| 结果窗「确定」 | 关闭结果窗和红框 |
| 托盘右键「配置」 | 修改百度 API 凭据、长按时间 |
| 托盘右键「退出」 | 退出程序 |

## 目录结构

```
rust/target/release/
├── floattrans.exe       ← 主程序（直接运行）
└── tesseract/           ← OCR 引擎（已内置）
    ├── tesseract.exe
    ├── *.dll
    └── tessdata/
        └── eng.traineddata
```

## 技术栈

| 层 | 技术 |
|----|------|
| 语言 | Rust (windows-rs 0.58) |
| OCR | Tesseract 5.0 + 自适应二值化预处理 |
| 翻译 | 百度翻译 API (from=auto, to=zh) |
| 二进制 | **~40MB**（含 OCR 引擎）— 无运行时依赖，clone 即用 |
