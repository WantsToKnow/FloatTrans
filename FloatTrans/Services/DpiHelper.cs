using System.Windows;

namespace FloatTrans;

/// <summary>
/// 全局鼠标钩子给出的是物理像素坐标;WPF 窗口的 Left/Top/Width 是逻辑坐标(DIP)。
/// app.manifest 声明为 System DPI awareness,故全机统一一个系统缩放,用本类换算。
/// </summary>
public static class DpiHelper
{
    public static double ScaleX { get; private set; } = 1.0;
    public static double ScaleY { get; private set; } = 1.0;

    public static void Init()
    {
        using var g = System.Drawing.Graphics.FromHwnd(IntPtr.Zero);
        ScaleX = g.DpiX / 96.0;
        ScaleY = g.DpiY / 96.0;
    }

    /// <summary>物理像素 → WPF 逻辑坐标</summary>
    public static Point PhysicalToLogical(double physX, double physY)
        => new(physX / ScaleX, physY / ScaleY);

    /// <summary>WPF 逻辑坐标 → 物理像素</summary>
    public static Point LogicalToPhysical(double logicalX, double logicalY)
        => new(logicalX * ScaleX, logicalY * ScaleY);
}
