using System.Windows;

namespace FloatTrans;

public partial class App : Application
{
    private SelectionController? _controller;
    private GlobalMouseHook? _hook;
    private System.Windows.Forms.NotifyIcon? _notifyIcon;
    private AppConfig? _config;
    private TranslateService? _translator;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        var config = AppConfig.Load();
        _config = config;

        _hook = new GlobalMouseHook();
        if (!_hook.Start())
        {
            MessageBox.Show("全局鼠标钩子安装失败,程序无法运行。", "FloatTrans", MessageBoxButton.OK, MessageBoxImage.Error);
            Shutdown();
            return;
        }

        var ball = new MainWindow();
        var ocr = new OcrService();
        var translator = new TranslateService(config);
        _translator = translator;
        _controller = new SelectionController(config, _hook, ball, ocr, translator);
        _controller.Start();
        ball.Show();

        SetupTrayIcon();

        // 异步测试翻译 API(翻译 "hello");失败则弹出配置窗口
        _ = TestTranslateAsync(translator);
    }

    private void SetupTrayIcon()
    {
        _notifyIcon = new System.Windows.Forms.NotifyIcon
        {
            Icon = System.Drawing.SystemIcons.Application,
            Text = "FloatTrans - 悬浮球截屏翻译(右键退出 / 双击配置)",
            Visible = true,
        };
        var menu = new System.Windows.Forms.ContextMenuStrip();
        menu.Items.Add("配置翻译 API...", null, (_, _) => OpenConfig());
        menu.Items.Add(new System.Windows.Forms.ToolStripSeparator());
        menu.Items.Add("退出 FloatTrans", null, (_, _) => Shutdown());
        _notifyIcon.ContextMenuStrip = menu;
        _notifyIcon.DoubleClick += (_, _) => OpenConfig();
    }

    private void OpenConfig()
    {
        if (_config is null || _translator is null) return;
        var win = new ConfigWindow(_config, _translator);
        win.Show();
    }

    private async Task TestTranslateAsync(TranslateService translator)
    {
        var result = await translator.TranslateEnToZhAsync("hello");
        if (TranslateService.IsSuccess(result)) return;

        // 未配置或测试失败:弹出配置窗口
        await Dispatcher.InvokeAsync(() =>
        {
            var win = new ConfigWindow(_config!, translator);
            win.Show();
        });
    }

    protected override void OnExit(ExitEventArgs e)
    {
        if (_notifyIcon is not null)
        {
            _notifyIcon.Visible = false;
            _notifyIcon.Dispose();
        }
        _controller?.Dispose();
        _hook?.Dispose();
        base.OnExit(e);
    }
}
