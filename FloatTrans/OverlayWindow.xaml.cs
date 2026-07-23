using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Interop;
using System.Windows.Media;

namespace FloatTrans;

/// <summary>
/// 框选覆盖层:跟随鼠标所在的单个显示器(取该屏真实 DPI),鼠标穿透。
/// Canvas 内用物理坐标绘制(相对该屏左上),RenderTransform 反缩放 → 显示与 CopyFromScreen 截图(物理)完全一致。
/// </summary>
public partial class OverlayWindow : Window
{
    private const int GWL_EXSTYLE = -20;
    private const int WS_EX_TRANSPARENT = 0x00000020;
    private const int WS_EX_LAYERED = 0x00080000;

    [DllImport("user32.dll", EntryPoint = "GetWindowLongPtrW", SetLastError = true)]
    private static extern IntPtr GetWindowLongPtr(IntPtr hWnd, int nIndex);

    [DllImport("user32.dll", EntryPoint = "SetWindowLongPtrW", SetLastError = true)]
    private static extern IntPtr SetWindowLongPtr(IntPtr hWnd, int nIndex, IntPtr dwNewLong);

    private double _physLeft, _physTop; // 当前覆盖屏的物理左上
    private bool _crossShown;

    public OverlayWindow()
    {
        InitializeComponent();
    }

    protected override void OnSourceInitialized(EventArgs e)
    {
        base.OnSourceInitialized(e);
        // 鼠标穿透:事件走全局钩子,窗口仅作画布
        var hwnd = new WindowInteropHelper(this).Handle;
        var ex = (int)GetWindowLongPtr(hwnd, GWL_EXSTYLE);
        SetWindowLongPtr(hwnd, GWL_EXSTYLE, new IntPtr(ex | WS_EX_TRANSPARENT | WS_EX_LAYERED));
    }

    /// <summary>定位到物理点所在屏并显示十字标(物理屏幕坐标)</summary>
    public void Begin(Point physicalStart)
    {
        MonitorHelper.TryGetMonitor((int)physicalStart.X, (int)physicalStart.Y,
            out var rc, out var sx, out var sy);
        _physLeft = rc.Left;
        _physTop = rc.Top;

        // overlay 覆盖该屏:DIP = 物理 / scale
        Left = rc.Left / sx;
        Top = rc.Top / sy;
        Width = (rc.Right - rc.Left) / sx;
        Height = (rc.Bottom - rc.Top) / sy;

        // Canvas 内坐标用物理值(相对该屏左上);RenderTransform 反缩放使其按物理像素显示
        Layer.RenderTransform = new ScaleTransform(1.0 / sx, 1.0 / sy);

        DrawCross(ToCanvas(physicalStart));
        Rect.Visibility = Visibility.Collapsed;
        _crossShown = true;
    }

    /// <summary>更新框选矩形(物理屏幕坐标)</summary>
    public void Update(Point physicalStart, Point physicalEnd)
    {
        if (!_crossShown) { DrawCross(ToCanvas(physicalStart)); _crossShown = true; }

        var a = ToCanvas(physicalStart);
        var b = ToCanvas(physicalEnd);
        var x = Math.Min(a.X, b.X);
        var y = Math.Min(a.Y, b.Y);
        var w = Math.Abs(a.X - b.X);
        var h = Math.Abs(a.Y - b.Y);
        Rect.Width = w;
        Rect.Height = h;
        Canvas.SetLeft(Rect, x);
        Canvas.SetTop(Rect, y);
        Rect.Visibility = (w > 1 && h > 1) ? Visibility.Visible : Visibility.Collapsed;
    }

    private Point ToCanvas(Point physical) => new(physical.X - _physLeft, physical.Y - _physTop);

    private void DrawCross(Point p)
    {
        const double r = 11;
        CrossH.X1 = p.X - r; CrossH.X2 = p.X + r; CrossH.Y1 = p.Y; CrossH.Y2 = p.Y;
        CrossV.X1 = p.X; CrossV.X2 = p.X; CrossV.Y1 = p.Y - r; CrossV.Y2 = p.Y + r;
        Canvas.SetLeft(CrossDot, p.X - CrossDot.Width / 2);
        Canvas.SetTop(CrossDot, p.Y - CrossDot.Height / 2);
        CrossH.Visibility = Visibility.Visible;
        CrossV.Visibility = Visibility.Visible;
        CrossDot.Visibility = Visibility.Visible;
    }

    public void Reset()
    {
        _crossShown = false;
        Rect.Visibility = Visibility.Collapsed;
        CrossH.Visibility = Visibility.Collapsed;
        CrossV.Visibility = Visibility.Collapsed;
        CrossDot.Visibility = Visibility.Collapsed;
    }
}
