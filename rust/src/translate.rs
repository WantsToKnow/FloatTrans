use crate::config::Config;
use md5::{Digest, Md5};
use std::time::{SystemTime, UNIX_EPOCH};

const ENDPOINT: &str = "https://fanyi-api.baidu.com/api/trans/vip/translate";

/// 英文/混合 → 中文。from=auto 自动检测,中文原样保留,英文翻译。
/// 所有错误信息均以 '[' 开头,可用 is_success 判断。
pub fn translate_to_zh(q: &str, cfg: &Config) -> String {
    if q.trim().is_empty() {
        return String::new();
    }
    if cfg.baidu_app_id.is_empty() || cfg.baidu_secret.is_empty() {
        return "[未配置] 请填入百度翻译 AppId / Secret".into();
    }

    let salt = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".into());

    let mut hasher = Md5::new();
    hasher.update(format!("{}{}{}{}", cfg.baidu_app_id, q, salt, cfg.baidu_secret).as_bytes());
    let sign_hex: String = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();

    let resp = ureq::post(ENDPOINT)
        .timeout(std::time::Duration::from_secs(15))
        .send_form(&[
            ("q", q),
            ("from", "auto"),
            ("to", "zh"),
            ("appid", &cfg.baidu_app_id),
            ("salt", &salt),
            ("sign", &sign_hex),
        ]);

    let body = match resp {
        Ok(r) => r.into_string().unwrap_or_default(),
        Err(e) => return format!("[翻译异常] {}", e),
    };

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
        if let Some(arr) = v["trans_result"].as_array() {
            let texts: Vec<String> = arr
                .iter()
                .filter_map(|i| i["dst"].as_str().map(String::from))
                .collect();
            let joined = texts.join("\n");
            if joined.is_empty() {
                return "[翻译结果为空]".into();
            }
            return joined;
        }
        if let Some(ec) = v["error_code"].as_str() {
            let msg = v["error_msg"].as_str().unwrap_or("");
            return format!("[百度错误 {}] {}", ec, msg);
        }
    }
    format!("[翻译失败] {}", body)
}

pub fn is_success(result: &str) -> bool {
    !result.is_empty() && !result.starts_with('[')
}
