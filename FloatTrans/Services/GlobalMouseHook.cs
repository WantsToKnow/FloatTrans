using System.Runtime.InteropServices;

namespace FloatTrans;

/// <summary>
/// 全局低级鼠标钩子(WH_MOUSE_LL)。
/// 用途:在悬浮球上左键长按 1.5s 后进入框选,鼠标可能已移出小球窗口;
/// 用全局钩子统一跟踪 MouseMove / LButtonUp,不依赖 WPF 鼠标捕获,跨窗口也可靠。
/// POINT 坐标为物理屏幕像素。
/// </summary>
public sealed class GlobalMouseHook : IDisposable
{
    private const int WH_MOUSE_LL = 14;
    private const int WM_MOUSEMOVE = 0x0200;
    private const int WM_LBUTTONDOWN = 0x0201;
    private const int WM_LBUTTONUP = 0x0202;

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int X; public int Y; }

    [StructLayout(LayoutKind.Sequential)]
    private struct MSLLHOOKSTRUCT
    {
        public POINT pt;
        public uint mouseData;
        public uint flags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    private delegate IntPtr LowLevelMouseProc(int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetWindowsHookEx(int idHook, LowLevelMouseProc lpfn, IntPtr hMod, uint dwThreadId);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnhookWindowsHookEx(IntPtr hhk);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    private static extern IntPtr GetModuleHandle(string? lpModuleName);

    [DllImport("user32.dll")]
    public static extern bool GetCursorPos(out POINT lpPoint);

    private IntPtr _hookId = IntPtr.Zero;
    private LowLevelMouseProc? _proc; // 必须保留委托引用,防止 GC 回收导致回调崩溃

    /// <summary>左键按下(物理坐标)</summary>
    public event Action<POINT>? LButtonDown;
    /// <summary>鼠标移动(物理坐标)</summary>
    public event Action<POINT>? MouseMove;
    /// <summary>左键抬起(物理坐标)</summary>
    public event Action<POINT>? LButtonUp;

    public bool Start()
    {
        _proc = HookProc;
        // GetModuleHandle(null) = 当前 EXE 模块句柄;低级钩子不注入其他进程,本进程模块即可。
        _hookId = SetWindowsHookEx(WH_MOUSE_LL, _proc!, GetModuleHandle(null), 0);
        return _hookId != IntPtr.Zero;
    }

    private IntPtr HookProc(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0)
        {
            var info = Marshal.PtrToStructure<MSLLHOOKSTRUCT>(lParam);
            switch (wParam.ToInt32())
            {
                case WM_LBUTTONDOWN: LButtonDown?.Invoke(info.pt); break;
                case WM_MOUSEMOVE: MouseMove?.Invoke(info.pt); break;
                case WM_LBUTTONUP: LButtonUp?.Invoke(info.pt); break;
            }
        }
        return CallNextHookEx(_hookId, nCode, wParam, lParam);
    }

    public void Dispose()
    {
        if (_hookId != IntPtr.Zero)
        {
            UnhookWindowsHookEx(_hookId);
            _hookId = IntPtr.Zero;
        }
    }
}
