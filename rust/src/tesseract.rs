use std::process::Command;

fn find_exe() -> Option<String> {
    let bundled = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("tesseract").join("tesseract.exe")));
    if let Some(ref p) = bundled {
        if p.exists() {
            if let Some(td) = p.parent().map(|d| d.join("tessdata")) {
                if td.exists() {
                    std::env::set_var("TESSDATA_PREFIX", td);
                }
            }
            return Some(p.to_string_lossy().to_string());
        }
    }
    if Command::new("tesseract").arg("--version").output().is_ok() {
        return Some("tesseract".into());
    }
    for base in &[
        r"C:\Program Files\Tesseract-OCR\tesseract.exe",
        r"C:\Program Files (x86)\Tesseract-OCR\tesseract.exe",
    ] {
        let p = std::path::Path::new(base);
        if p.exists() {
            if let Some(td) = p.parent().map(|d| d.join("tessdata")) {
                if td.exists() {
                    std::env::set_var("TESSDATA_PREFIX", td);
                }
            }
            return Some(base.to_string());
        }
    }
    None
}

static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

pub fn available() -> bool {
    *AVAILABLE.get_or_init(|| {
        find_exe().map(|exe| {
            let check = Command::new(&exe).arg("--list-langs").output().ok();
            if let Some(out) = check {
                let s = String::from_utf8_lossy(&out.stdout);
                return s.lines().any(|l| l.trim() == "eng");
            }
            false
        }).unwrap_or(false)
    })
}

pub fn recognize(bgra: &[u8], w: i32, h: i32) -> Option<String> {
    if !available() { return None; }
    let exe = find_exe()?;

    let tmp = std::env::temp_dir().join(format!("floattrans_ocr_{}.png", std::process::id()));
    let img = to_png(bgra, w, h)?;
    std::fs::write(&tmp, &img).ok()?;

    let out = Command::new(&exe)
        .arg(&tmp).arg("stdout")
        .arg("-l").arg("eng")
        .arg("--psm").arg("6")
        .output().ok()?;

    let _ = std::fs::remove_file(&tmp);

    if out.status.success() {
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !text.is_empty() { return Some(text); }
    }
    None
}

fn to_png(bgra: &[u8], w: i32, h: i32) -> Option<Vec<u8>> {
    use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapEncoder, BitmapPixelFormat};
    use windows::Storage::Streams::{DataReader, InMemoryRandomAccessStream};

    let stream = InMemoryRandomAccessStream::new().ok()?;
    let encoder = BitmapEncoder::CreateAsync(BitmapEncoder::PngEncoderId().ok()?, &stream).ok()?.get().ok()?;
    encoder.SetPixelData(BitmapPixelFormat::Bgra8, BitmapAlphaMode::Premultiplied, w as u32, h as u32, 96.0, 96.0, bgra).ok()?;
    encoder.FlushAsync().ok()?.get().ok()?;
    stream.Seek(0).ok()?;

    let len = stream.Size().ok()? as usize;
    let mut buf = vec![0u8; len];
    let reader = DataReader::CreateDataReader(&stream).ok()?;
    reader.LoadAsync(len as u32).ok()?.get().ok()?;
    reader.ReadBytes(&mut buf).ok()?;
    Some(buf)
}
