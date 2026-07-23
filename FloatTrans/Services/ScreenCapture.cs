using System.Drawing;
using System.Drawing.Imaging;

namespace FloatTrans;

/// <summary>屏幕截图(物理像素坐标)。</summary>
public static class ScreenCapture
{
    /// <summary>截取指定物理矩形区域。注意:CopyFromScreen 用物理像素,与全局钩子坐标一致。</summary>
    public static Bitmap Capture(int x, int y, int width, int height)
    {
        if (width <= 0 || height <= 0)
            throw new ArgumentException($"截图尺寸必须为正,当前 w={width} h={height}");

        var bmp = new Bitmap(width, height, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(bmp))
        {
            g.CopyFromScreen(x, y, 0, 0, new Size(width, height), CopyPixelOperation.SourceCopy);
        }
        return bmp;
    }
}
