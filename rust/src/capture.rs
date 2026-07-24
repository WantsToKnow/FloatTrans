use windows::Win32::Graphics::Gdi::*;

pub struct Captured {
    pub width: i32,
    pub height: i32,
    pub bgra: Vec<u8>,
}

/// 截取指定物理屏幕矩形(物理像素坐标,与全局钩子一致)。
pub fn capture(x: i32, y: i32, w: i32, h: i32) -> windows::core::Result<Captured> {
    if w <= 0 || h <= 0 {
        return Err(windows::core::Error::from(windows::core::HRESULT(-1)));
    }
    unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(screen);
        let bmp = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(mem, bmp);
        let _ = BitBlt(mem, 0, 0, w, h, screen, x, y, SRCCOPY);

        let mut bi: BITMAPINFO = Default::default();
        bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = w;
        bi.bmiHeader.biHeight = -h; // top-down
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = 0; // BI_RGB

        let mut bgra = vec![0u8; (w * h * 4) as usize];
        GetDIBits(
            mem,
            bmp,
            0,
            h as u32,
            Some(bgra.as_mut_ptr() as *mut core::ffi::c_void),
            &mut bi,
            DIB_RGB_COLORS,
        );

        SelectObject(mem, old);
        let _ = DeleteObject(bmp);
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        Ok(Captured {
            width: w,
            height: h,
            bgra,
        })
    }
}
