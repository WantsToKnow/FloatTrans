use crate::tesseract;

pub const OCR_ERROR: &str = "[OCR 引擎未就绪]\n请确保 floattrans.exe 同目录下有 tesseract 文件夹\n内含 tesseract.exe + tessdata/eng.traineddata";

#[derive(Clone)]
pub struct Ocr;

impl Ocr {
    pub fn new() -> Self { Ocr }

    pub fn available(&self) -> bool { tesseract::available() }

    pub fn recognize(&self, bgra: &[u8], w: i32, h: i32) -> Result<String, String> {
        tesseract::recognize(bgra, w, h).ok_or_else(|| OCR_ERROR.to_string())
    }
}
