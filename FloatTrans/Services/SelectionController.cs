using System.Windows;
using System.Windows.Threading;

namespace FloatTrans;

/// <summary>
/// 核心状态机:协调悬浮球 / 全局鼠标钩子 / 框选覆盖层 / 截图 / OCR / 翻译 / 结果窗口。
///
/// 交互:
///   - 鼠标在悬浮球上左键按下 → Pressing,启动 1.5s 计时。
///   - 1.5s 内移动超过阈值 → Dragging,拖动悬浮球。
///   - 1.5s 内松开 → Idle(短按,忽略)。
///   - 1.5s 到达(左键仍按)→ Selecting:显示十字标,进入框选。
///   - Selecting 中移动 → 实时绘制矩形;松开左键 → 截图矩形 → OCR → 翻译 → 显示结果。
///
/// 鼠标事件由 GlobalMouseHook 在 UI 线程回调。
/// OnUp 把重活派发到 Dispatcher,使低级钩子回调立即返回(避免超 LowLevelHooksTimeout 被系统摘钩)。
/// ProcessAsync 截图前先隐藏覆盖层/悬浮球/旧结果窗并等待 DWM 重新合成,避免它们被 CopyFromScreen 截入。
/// </summary>
public sealed class SelectionController : IDisposable
{
    private enum State { Idle, Pressing, Dragging, Selecting }

    private readonly AppConfig _config;
    private readonly GlobalMouseHook _hook;
    private readonly MainWindow _ball;
    private readonly OcrService _ocr;
    private readonly TranslateService _translator;
    private readonly OverlayWindow _overlay;

    private State _state = State.Idle;
    private GlobalMouseHook.POINT _downPoint;   // 物理坐标
    private DispatcherTimer? _holdTimer;
    private Point _dragOffsetPhys;              // 球心 − 按下点(物理)
    private ResultWindow? _lastResult;
    private CancellationTokenSource? _cts;
    private const double DragThreshold = 5.0;   // px

    public SelectionController(AppConfig config, GlobalMouseHook hook, MainWindow ball,
                               OcrService ocr, TranslateService translator)
    {
        _config = config;
        _hook = hook;
        _ball = ball;
        _ocr = ocr;
        _translator = translator;
        _overlay = new OverlayWindow();
    }

    public void Start()
    {
        _hook.LButtonDown += OnDown;
        _hook.MouseMove += OnMove;
        _hook.LButtonUp += OnUp;
    }

    private bool IsPointOnBall(GlobalMouseHook.POINT p)
    {
        var r = _ball.GetPhysicalRect();
        return p.X >= r.Left && p.X <= r.Right && p.Y >= r.Top && p.Y <= r.Bottom;
    }

    private void OnDown(GlobalMouseHook.POINT p)
    {
        if (_state != State.Idle) return;
        if (!IsPointOnBall(p)) return; // 仅在悬浮球上按下才进入流程

        _downPoint = p;
        _state = State.Pressing;

        _holdTimer?.Stop();
        _holdTimer = new DispatcherTimer
        {
            Interval = TimeSpan.FromMilliseconds(Math.Max(200, _config.HoldMilliseconds))
        };
        _holdTimer.Tick += (_, _) =>
        {
            _holdTimer!.Stop();
            if (_state == State.Pressing) EnterSelecting();
        };
        _holdTimer.Start();
    }

    private void OnMove(GlobalMouseHook.POINT p)
    {
        switch (_state)
        {
            case State.Pressing:
            {
                double dx = p.X - _downPoint.X, dy = p.Y - _downPoint.Y;
                if (dx * dx + dy * dy > DragThreshold * DragThreshold)
                {
                    _holdTimer!.Stop();
                    var r = _ball.GetPhysicalRect();
                    _dragOffsetPhys = new Point((r.Left + r.Right) / 2.0 - _downPoint.X,
                                                (r.Top + r.Bottom) / 2.0 - _downPoint.Y);
                    _state = State.Dragging;
                    MoveBall(p);
                }
                break;
            }
            case State.Dragging:
                MoveBall(p);
                break;
            case State.Selecting:
                _overlay.Update(ToPoint(_downPoint), ToPoint(p));
                break;
        }
    }

    private void OnUp(GlobalMouseHook.POINT p)
    {
        switch (_state)
        {
            case State.Pressing:
                _holdTimer!.Stop();
                _state = State.Idle; // 短按:忽略
                break;
            case State.Dragging:
                _state = State.Idle;
                break;
            case State.Selecting:
            {
                var start = _downPoint;
                var end = p;
                _state = State.Idle;
                // 取消上一次仍在进行的 OCR/翻译(释放 QPS 门、停止写旧窗口)
                _cts?.Cancel();
                // 重活派发到 Dispatcher,使低级钩子回调立即返回(避免 LowLevelHooksTimeout 摘钩)
                Application.Current.Dispatcher.BeginInvoke(new Action(() => _ = ProcessAsync(start, end)));
                break;
            }
        }
    }

    private void MoveBall(GlobalMouseHook.POINT p)
        => _ball.SetCenterPhysical(p.X + (int)_dragOffsetPhys.X, p.Y + (int)_dragOffsetPhys.Y);

    private void EnterSelecting()
    {
        _state = State.Selecting;
        // 框选起点用"当前"鼠标位置(长按触发瞬间指针所在处),而非最初按下点
        GlobalMouseHook.GetCursorPos(out var cur);
        _downPoint = cur;
        _overlay.Reset();
        _overlay.Show();
        _overlay.Begin(ToPoint(_downPoint));
    }

    private static Point ToPoint(GlobalMouseHook.POINT p) => new(p.X, p.Y);

    private async Task ProcessAsync(GlobalMouseHook.POINT start, GlobalMouseHook.POINT end)
    {
        int x = Math.Min(start.X, end.X);
        int y = Math.Min(start.Y, end.Y);
        int w = Math.Abs(start.X - end.X);
        int h = Math.Abs(start.Y - end.Y);
        if (w < 5 || h < 5) return;

        _cts?.Dispose();
        _cts = new CancellationTokenSource();
        var token = _cts.Token;

        ResultWindow? result = null;
        try
        {
            // 关闭旧结果窗口 + 隐藏覆盖层与悬浮球,等待 DWM 重新合成后再截图,
            // 否则这些窗口会被 CopyFromScreen 一起截入,污染 OCR。
            _lastResult?.Close();
            _overlay.Hide();
            _overlay.Reset();
            _ball.Hide();
            await Task.Delay(100);
            token.ThrowIfCancellationRequested();

            using var bmp = ScreenCapture.Capture(x, y, w, h);
            _ball.Show(); // 截图完成,恢复悬浮球(后续 OCR/翻译时球可见)
            token.ThrowIfCancellationRequested();

            // 结果窗口在截图之后创建,避免被截入
            result = new ResultWindow();
            result.Closed += (_, _) => { if (_lastResult == result) _lastResult = null; };
            _lastResult = result;
            result.ShowLoading(x, y, w, h);
            result.Show();

            var en = await _ocr.RecognizeAsync(bmp);
            token.ThrowIfCancellationRequested();
            result.SetEnglish(en);

            if (en.StartsWith("[error]") || string.IsNullOrWhiteSpace(en))
                result.SetChinese(string.IsNullOrWhiteSpace(en) ? "(未识别到文字)" : en);
            else
                result.SetChinese(await _translator.TranslateEnToZhAsync(en, token));
        }
        catch (OperationCanceledException)
        {
            // 被新一次框选取消,静默关闭本次结果窗口
            result?.Close();
        }
        catch (Exception ex)
        {
            try { result?.SetChinese("[异常] " + ex.Message); } catch { }
        }
        finally
        {
            _ball.Show(); // 幂等保护:确保悬浮球恢复
        }
    }

    public void Dispose()
    {
        _hook.LButtonDown -= OnDown;
        _hook.MouseMove -= OnMove;
        _hook.LButtonUp -= OnUp;
        _holdTimer?.Stop();
        _cts?.Cancel();
        _cts?.Dispose();
        _overlay.Close();
    }
}
