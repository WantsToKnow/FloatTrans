using System.IO;
using System.Text.Json;

namespace FloatTrans;

/// <summary>应用配置,存放于 %AppData%\FloatTrans\config.json</summary>
public sealed class AppConfig
{
    /// <summary>百度翻译 AppId</summary>
    public string BaiduAppId { get; set; } = "";

    /// <summary>百度翻译密钥 Secret</summary>
    public string BaiduSecret { get; set; } = "";

    /// <summary>长按触发框选的阈值(毫秒)</summary>
    public int HoldMilliseconds { get; set; } = 500;

    /// <summary>悬浮球直径(逻辑像素 DIP)</summary>
    public double BallSize { get; set; } = 54;

    /// <summary>悬浮球默认透明度 0~1</summary>
    public double BallOpacity { get; set; } = 0.55;

    private static string ConfigDir =>
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "FloatTrans");

    private static string ConfigPath => Path.Combine(ConfigDir, "config.json");

    public static AppConfig Load()
    {
        try
        {
            if (File.Exists(ConfigPath))
            {
                var json = File.ReadAllText(ConfigPath);
                return JsonSerializer.Deserialize<AppConfig>(json) ?? CreateDefault(writeFile: false);
            }
        }
        catch (JsonException)
        {
            // JSON 损坏:返回默认但不写盘,避免覆盖可能有效的原文件
            return CreateDefault(writeFile: false);
        }
        catch (IOException)
        {
            // 瞬时 I/O 错误(杀软/备份/第二实例短暂独占):返回内存默认,绝不写盘,以免删除用户凭据
            return new AppConfig();
        }
        catch (UnauthorizedAccessException)
        {
            return new AppConfig();
        }
        // 文件不存在:首次运行,创建默认并写盘
        return CreateDefault(writeFile: true);
    }

    private static AppConfig CreateDefault(bool writeFile)
    {
        var cfg = new AppConfig();
        if (writeFile) cfg.Save();
        return cfg;
    }

    public void Save()
    {
        Directory.CreateDirectory(ConfigDir);
        var json = JsonSerializer.Serialize(this, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(ConfigPath, json);
    }

    /// <summary>配置文件完整路径,用于在 UI 中提示用户编辑</summary>
    public static string GetConfigFilePath() => ConfigPath;
}
