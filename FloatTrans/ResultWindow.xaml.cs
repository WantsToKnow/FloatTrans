using System.Windows;

namespace FloatTrans;

/// <summary>翻译结果窗口:显示 OCR 英文 + 百度翻译中文,可一键复制。</summary>
public partial class ResultWindow : Window
{
    public ResultWindow()
    {
        InitializeComponent();
    }

    public void ShowLoading(int x, int y, int w, int h)
    {
        Title = $"识别中… 区域 {w}×{h} @({x},{y})";
        EnBox.Text = "正在识别文字…";
        ZhBox.Text = "正在翻译…";
    }

    public void SetEnglish(string text) { EnBox.Text = text; Title = "翻译结果"; }
    public void SetChinese(string text) { ZhBox.Text = text; }

    private void CopyZhClick(object sender, RoutedEventArgs e)
    {
        try { if (!string.IsNullOrEmpty(ZhBox.Text)) Clipboard.SetText(ZhBox.Text); } catch { }
    }

    private void CopyEnClick(object sender, RoutedEventArgs e)
    {
        try { if (!string.IsNullOrEmpty(EnBox.Text)) Clipboard.SetText(EnBox.Text); } catch { }
    }

    private void CloseClick(object sender, RoutedEventArgs e) => Close();
}
