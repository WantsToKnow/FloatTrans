using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Interop;

namespace FloatTrans;

/// <summary>桌面悬浮球:无边框、置顶、半透明、不穿透。拖动用 SetWindowPos 物理定位(多 DPI 屏通用)。</summary>
public partial class MainWindow : Window
{
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }

    private static readonly IntPtr HWND_TOPMOST = new(-1);
    private const uint SWP_NOSIZE = 0x0001;
    private const uint SWP_NOACTIVATE = 0x0010;

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);

    public MainWindow()
    {
        InitializeComponent();
        Loaded += OnLoaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        // 初始位置:主屏工作区右下角内侧
        var workArea = SystemParameters.WorkArea;
        Left = workArea.Right - Width - 20;
        Top = workArea.Bottom - Height - 20;
    }

    /// <summary>球的物理屏幕矩形(供全局钩子做命中判断)</summary>
    public RECT GetPhysicalRect()
    {
        var hwnd = new WindowInteropHelper(this).Handle;
        GetWindowRect(hwnd, out var r);
        return r;
    }

    /// <summary>按物理坐标设置球心位置:用 SetWindowPos 物理定位,绕过 DPI 换算,多屏通用。</summary>
    public void SetCenterPhysical(int physX, int physY)
    {
        var hwnd = new WindowInteropHelper(this).Handle;
        GetWindowRect(hwnd, out var r);
        int w = r.Right - r.Left;
        int h = r.Bottom - r.Top;
        SetWindowPos(hwnd, HWND_TOPMOST, physX - w / 2, physY - h / 2, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
    }
}
