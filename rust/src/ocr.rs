use windows::core::*;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapDecoder, BitmapEncoder, BitmapPixelFormat};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::InMemoryRandomAccessStream;

#[derive(Clone)]
pub struct Ocr {
    en: Option<OcrEngine>,
}

fn lang(tag: &str) -> Option<Language> {
    Language::CreateLanguage(&windows::core::HSTRING::from(tag)).ok()
}

pub const OCR_ERROR: &str = "[OCR 引擎未就绪]\n请安装英文 OCR 语言包:\n设置→时间和语言→语言→添加语言→\nEnglish (United States)→勾选\"OCR\"重试";

impl Ocr {
    pub fn new() -> Self {
        let en = lang("en").and_then(|l| OcrEngine::TryCreateFromLanguage(&l).ok());
        Ocr { en }
    }

    pub fn available(&self) -> bool { self.en.is_some() }

    pub fn recognize(&self, bgra: &[u8], w: i32, h: i32) -> Result<String> {
        let en = self.en.as_ref().ok_or_else(|| Error::from(HRESULT(-1)))?;
        let (bgra2, w2, h2) = preprocess(bgra, w, h);

        let stream = InMemoryRandomAccessStream::new()?;
        let encoder = BitmapEncoder::CreateAsync(BitmapEncoder::PngEncoderId()?, &stream)?.get()?;
        encoder.SetPixelData(
            BitmapPixelFormat::Bgra8,
            BitmapAlphaMode::Premultiplied,
            w2 as u32,
            h2 as u32,
            96.0,
            96.0,
            &bgra2,
        )?;
        encoder.FlushAsync()?.get()?;
        stream.Seek(0)?;

        let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
        let sb = decoder.GetSoftwareBitmapAsync()?.get()?;
        Ok(en.RecognizeAsync(&sb)?.get()?.Text()?.to_string().trim().to_string())
    }
}

/// 预处理: 放大 2x(nearest) + 对比度 1.3x
fn preprocess(bgra: &[u8], w: i32, h: i32) -> (Vec<u8>, i32, i32) {
    const SCALE: i32 = 2;
    const C: f32 = 1.3;
    let t = (1.0 - C) * 128.0;
    let nw = w * SCALE;
    let nh = h * SCALE;
    let mut out = vec![0u8; (nw * nh * 4) as usize];
    for y in 0..nh {
        for x in 0..nw {
            let sx = (x / SCALE) as usize;
            let sy = (y / SCALE) as usize;
            let si = (sy * w as usize + sx) * 4;
            let di = (y as usize * nw as usize + x as usize) * 4;
            for ch in 0..3usize {
                let v = bgra[si + ch] as f32;
                out[di + ch] = (v * C + t).clamp(0.0, 255.0) as u8;
            }
            out[di + 3] = bgra[si + 3];
        }
    }
    (out, nw, nh)
}
