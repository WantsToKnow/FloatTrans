using System.Runtime.InteropServices;

namespace FloatTrans;

/// <summary>显示器信息辅助:按物理坐标查找所在屏的物理边界与 DPI 缩放(支持多屏混合 DPI)。</summary>
public static class MonitorHelper
{
    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int X; public int Y; }

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }

    [StructLayout(LayoutKind.Sequential)]
    private struct MONITORINFO
    {
        public uint cbSize;
        public RECT rcMonitor;
        public RECT rcWork;
        public uint dwFlags;
    }

    [DllImport("user32.dll")]
    private static extern IntPtr MonitorFromPoint(POINT pt, uint dwFlags);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetMonitorInfo(IntPtr hMonitor, ref MONITORINFO lpmi);

    [DllImport("shcore.dll")]
    private static extern int GetDpiForMonitor(IntPtr hMonitor, int dpiType, out uint dpiX, out uint dpiY);

    private const uint MONITOR_DEFAULTTONEAREST = 2;
    private const int MDT_EFFECTIVE_DPI = 0;

    /// <summary>获取物理点所在屏的物理边界与 DPI 缩放。失败时 scale=1 兜底。</summary>
    public static bool TryGetMonitor(int physX, int physY,
        out RECT rcMonitor, out double scaleX, out double scaleY)
    {
        rcMonitor = default;
        scaleX = scaleY = 1.0;
        var hm = MonitorFromPoint(new POINT { X = physX, Y = physY }, MONITOR_DEFAULTTONEAREST);
        if (hm == IntPtr.Zero) return false;
        var info = new MONITORINFO { cbSize = (uint)Marshal.SizeOf<MONITORINFO>() };
        if (!GetMonitorInfo(hm, ref info)) return false;
        rcMonitor = info.rcMonitor;
        if (GetDpiForMonitor(hm, MDT_EFFECTIVE_DPI, out var dpix, out var dpiy) == 0 && dpix > 0)
        {
            scaleX = dpix / 96.0;
            scaleY = dpiy / 96.0;
        }
        return true;
    }
}
