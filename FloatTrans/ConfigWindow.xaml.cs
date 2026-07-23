using System.Diagnostics;
using System.Windows;
using System.Windows.Documents;
using System.Windows.Navigation;

namespace FloatTrans;

/// <summary>百度翻译 API 配置窗口:输入 AppId/Secret,保存并测试。</summary>
public partial class ConfigWindow : Window
{
    private readonly AppConfig _config;
    private readonly TranslateService _translator;

    public ConfigWindow(AppConfig config, TranslateService translator)
    {
        InitializeComponent();
        _config = config;
        _translator = translator;
        AppIdBox.Text = _config.BaiduAppId;
        SecretBox.Text = _config.BaiduSecret;
    }

    private void OpenUrl(object sender, RequestNavigateEventArgs e)
    {
        try { Process.Start(new ProcessStartInfo(e.Uri.AbsoluteUri) { UseShellExecute = true }); } catch { }
        e.Handled = true;
    }

    private async void SaveTestClick(object sender, RoutedEventArgs e)
    {
        _config.BaiduAppId = AppIdBox.Text.Trim();
        _config.BaiduSecret = SecretBox.Text.Trim();
        _config.Save();

        StatusText.Foreground = System.Windows.Media.Brushes.DimGray;
        StatusText.Text = "正在测试…";
        var result = await _translator.TranslateEnToZhAsync("hello");
        if (TranslateService.IsSuccess(result))
        {
            StatusText.Foreground = System.Windows.Media.Brushes.Green;
            StatusText.Text = "✓ 测试成功:「hello」→ " + result;
            await Task.Delay(900);
            Close();
        }
        else
        {
            StatusText.Foreground = System.Windows.Media.Brushes.Crimson;
            StatusText.Text = "✗ " + result;
        }
    }

    private void LaterClick(object sender, RoutedEventArgs e) => Close();
}
