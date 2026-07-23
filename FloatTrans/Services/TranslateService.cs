using System.Collections.Generic;
using System.Net.Http;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace FloatTrans;

/// <summary>
/// 英文 → 中文翻译:百度翻译通用翻译 API(免费标准版,QPS=1)。
/// 文档:https://fanyi-api.baidu.com/doc/21
/// 签名 sign = MD5(appid + q + salt + secret),小写十六进制(用原始未编码 q)。
/// 用 POST + 表单提交,避免长文本触发 URL 414 URI Too Long。
/// </summary>
public sealed class TranslateService
{
    private const string Endpoint = "https://fanyi-api.baidu.com/api/trans/vip/translate";

    private readonly AppConfig _config;
    private static readonly HttpClient _http = new HttpClient();
    private static readonly SemaphoreSlim _gate = new SemaphoreSlim(1, 1);
    private static DateTimeOffset _lastCallUtc = DateTimeOffset.MinValue;

    public TranslateService(AppConfig config) => _config = config;

    public async Task<string> TranslateEnToZhAsync(string q, CancellationToken token = default)
    {
        if (string.IsNullOrWhiteSpace(q)) return string.Empty;

        if (string.IsNullOrWhiteSpace(_config.BaiduAppId) || string.IsNullOrWhiteSpace(_config.BaiduSecret))
            return "[未配置] 请在配置文件填入百度翻译 AppId / Secret 后重试。";

        // 百度标准版 QPS=1,节流:距上次调用至少间隔 1.1s
        await _gate.WaitAsync(token);
        try
        {
            var elapsed = DateTimeOffset.UtcNow - _lastCallUtc;
            var wait = TimeSpan.FromSeconds(1.1) - elapsed;
            if (wait > TimeSpan.Zero) await Task.Delay(wait, token);
            _lastCallUtc = DateTimeOffset.UtcNow;

            var salt = Guid.NewGuid().ToString("N");
            var sign = Md5Hex(_config.BaiduAppId + q + salt + _config.BaiduSecret);

            // POST + application/x-www-form-urlencoded;q 放在请求体,sign 仍用原始未编码 q 计算
            var form = new Dictionary<string, string>
            {
                { "q", q },
                { "from", "en" },
                { "to", "zh" },
                { "appid", _config.BaiduAppId },
                { "salt", salt },
                { "sign", sign },
            };
            using var content = new FormUrlEncodedContent(form);

            // 外部 token(新框选取消)+ 15s 超时
            using var cts = CancellationTokenSource.CreateLinkedTokenSource(token);
            cts.CancelAfter(TimeSpan.FromSeconds(15));
            using var resp = await _http.PostAsync(Endpoint, content, cts.Token);
            var json = await resp.Content.ReadAsStringAsync(cts.Token);

            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            if (root.TryGetProperty("trans_result", out var arr) && arr.ValueKind == JsonValueKind.Array)
            {
                var sb = new StringBuilder();
                foreach (var item in arr.EnumerateArray())
                {
                    if (item.TryGetProperty("dst", out var dst))
                        sb.AppendLine(dst.GetString());
                }
                var text = sb.ToString().TrimEnd();
                return string.IsNullOrEmpty(text) ? "[翻译结果为空]" : text;
            }

            if (root.TryGetProperty("error_code", out var ec))
            {
                var msg = root.TryGetProperty("error_msg", out var em) ? em.GetString() : "";
                return $"[百度错误 {ec.GetString()}] {msg}";
            }

            return "[翻译失败] " + json;
        }
        catch (OperationCanceledException) when (token.IsCancellationRequested)
        {
            throw; // 被新框选取消,向上抛出让 SelectionController 静默处理
        }
        catch (Exception ex)
        {
            return "[翻译异常] " + ex.Message;
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>判断翻译返回是否为成功结果(所有错误信息均以 '[' 开头)</summary>
    public static bool IsSuccess(string result)
        => !string.IsNullOrEmpty(result) && !result.StartsWith("[");

    private static string Md5Hex(string s)
    {
        var bytes = MD5.HashData(Encoding.UTF8.GetBytes(s));
        var sb = new StringBuilder(bytes.Length * 2);
        foreach (var b in bytes) sb.Append(b.ToString("x2"));
        return sb.ToString();
    }
}
