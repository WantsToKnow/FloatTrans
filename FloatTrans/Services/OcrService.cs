using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.IO;
using Windows.Globalization;
using Windows.Graphics.Imaging;
using Windows.Media.Ocr;
using Windows.Storage.Streams;

namespace FloatTrans;

/// <summary>
/// 本地 OCR:使用 Windows.Media.Ocr(Win10/11 自带,离线、免费)。
/// 需系统已安装英文 OCR 语言包,否则 IsAvailable=false。
/// 预处理:放大 2x + 对比度增强,提升小字/低分辨率截图的识别率。
/// </summary>
public sealed class OcrService
{
    private readonly OcrEngine? _engine;

    public bool IsAvailable => _engine is not null;

    public OcrService()
    {
        var lang = new Language("en");
        _engine = OcrEngine.TryCreateFromLanguage(lang);
    }

    public async Task<string> RecognizeAsync(Bitmap src)
    {
        if (_engine is null)
            return "[error] 未安装英文 OCR 语言包。\n请到:设置 → 时间和语言 → 语言 → 添加语言 → 选 English (United States) → 勾选“OCR”";

        // 预处理:放大 2x(双三次)+ 对比度 1.3x,提升小字识别率
        using var enlarged = new Bitmap(src.Width * 2, src.Height * 2, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(enlarged))
        {
            g.InterpolationMode = InterpolationMode.HighQualityBicubic;
            g.PixelOffsetMode = PixelOffsetMode.HighQuality;
            g.SmoothingMode = SmoothingMode.HighQuality;

            const float c = 1.3f;               // 对比度
            float t = (1.0f - c) * 0.5f;        // 对比度平移
            var cm = new ColorMatrix(new[]
            {
                new float[] { c, 0, 0, 0, 0 },
                new float[] { 0, c, 0, 0, 0 },
                new float[] { 0, 0, c, 0, 0 },
                new float[] { 0, 0, 0, 1, 0 },
                new float[] { t, t, t, 0, 1 },
            });
            using var ia = new ImageAttributes();
            ia.SetColorMatrix(cm);
            g.DrawImage(src, new Rectangle(0, 0, enlarged.Width, enlarged.Height),
                0, 0, src.Width, src.Height, GraphicsUnit.Pixel, ia);
        }

        // Bitmap → PNG → InMemoryRandomAccessStream → BitmapDecoder → SoftwareBitmap → OCR
        using var ms = new MemoryStream();
        enlarged.Save(ms, ImageFormat.Png);
        var bytes = ms.ToArray();

        using var ras = new InMemoryRandomAccessStream();
        using (var dw = new DataWriter(ras.GetOutputStreamAt(0)))
        {
            dw.WriteBytes(bytes);
            await dw.StoreAsync();
            await dw.FlushAsync();
            dw.DetachStream(); // 防止 DataWriter Dispose 关闭底层 ras
        }
        ras.Seek(0);

        var decoder = await BitmapDecoder.CreateAsync(ras);
        using var softwareBitmap = await decoder.GetSoftwareBitmapAsync();
        var result = await _engine.RecognizeAsync(softwareBitmap);
        return result.Text ?? string.Empty;
    }
}
