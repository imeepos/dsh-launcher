//! release-platform HTTP 客户端（DESIGN-TOOLS.md §2）。
//! 内网 http 部署；ureq 关默认特性（http-only），后续 https 再开 tls feature。

use crate::registry::{RpAuth, RpAuthMode, RpSettings};
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::time::Duration;

pub const DEFAULT_BASE_URL: &str = "http://192.168.0.102:38080";
const API_TIMEOUT: Duration = Duration::from_secs(15);
const GRANT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct RpClient {
    agent: ureq::Agent,
    base_url: String,
    headers: Vec<(String, String)>,
}

/// 查询值百分号编码（保守：非安全字符全转 %XX，空格转 %20）。
pub fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// 本机 os 映射为平台制品命名：macos→darwin。
pub fn local_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    }
}

/// 本机 arch 映射：aarch64→arm64，x86_64→amd64。
pub fn local_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "amd64",
        other => other,
    }
}

/// 列表信封解包：{items:[..]} 取 items；裸数组（如 GET /v1/products）原样返回。
pub fn envelope_items(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(arr) => arr.clone(),
        other => other
            .get("items")
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default(),
    }
}

/// 拼查询串；空值跳过。
pub fn query_url(base: &str, path: &str, query: &[(&str, &str)]) -> String {
    let mut url = format!("{}{}", base.trim_end_matches('/'), path);
    let pairs: Vec<String> = query
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| format!("{}={}", k, urlencode(v)))
        .collect();
    if !pairs.is_empty() {
        url.push('?');
        url.push_str(&pairs.join("&"));
    }
    url
}

fn auth_headers(auth: &RpAuth, jwt: Option<&str>) -> Vec<(String, String)> {
    match auth.mode {
        RpAuthMode::Bearer => auth
            .token
            .as_deref()
            .filter(|t| !t.is_empty())
            .map(|t| vec![("Authorization".into(), format!("Bearer {t}"))])
            .unwrap_or_default(),
        RpAuthMode::Password => jwt
            .filter(|j| !j.is_empty())
            .map(|j| vec![("Authorization".into(), format!("Bearer {j}"))])
            .unwrap_or_default(),
        RpAuthMode::DevHeaders => {
            let mut h = Vec::new();
            if let Some(t) = auth.tenant.as_deref().filter(|s| !s.is_empty()) {
                h.push(("X-Tenant-ID".into(), t.into()));
            }
            if let Some(s) = auth.subject.as_deref().filter(|s| !s.is_empty()) {
                h.push(("X-Subject".into(), s.into()));
            }
            h
        }
    }
}

impl RpClient {
    /// password 模式需先经 password_grant 换 JWT 传入；其余模式用配置内凭证。
    pub fn new(settings: &RpSettings, jwt: Option<&str>) -> Self {
        let agent = ureq::AgentBuilder::new().timeout(API_TIMEOUT).build();
        let headers = settings
            .auth
            .as_ref()
            .map(|a| auth_headers(a, jwt))
            .unwrap_or_default();
        Self {
            agent,
            base_url: settings.base_url.trim_end_matches('/').into(),
            headers,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// GET JSON；平台错误码透传（DESIGN-TOOLS.md §5 红线）。
    pub fn get_json(&self, path: &str, query: &[(&str, &str)]) -> Result<Value, String> {
        let url = query_url(&self.base_url, path, query);
        let mut req = self.agent.get(&url);
        for (k, v) in &self.headers {
            req = req.set(k, v);
        }
        let resp = req.call().map_err(http_error)?;
        resp.into_json::<Value>()
            .map_err(|e| format!("响应 JSON 解析失败: {e}"))
    }
}

/// 平台错误透传：状态码 + body 内 code/message；网络错误原样。
fn http_error(err: ureq::Error) -> String {
    if let ureq::Error::Status(code, resp) = err {
        let body = resp.into_string().unwrap_or_default();
        return format!("平台返回 {code}: {body}");
    }
    format!("请求失败: {err}")
}

/// OAuth password grant 换 JWT（issuer_url 形如 http://host:port/oauth）。
pub fn password_grant(issuer_url: &str, username: &str, password: &str) -> Result<String, String> {
    let url = format!("{}/token", issuer_url.trim_end_matches('/'));
    let form = format!(
        "grant_type=password&username={}&password={}",
        urlencode(username),
        urlencode(password)
    );
    let resp = ureq::post(&url)
        .timeout(GRANT_TIMEOUT)
        .send_string(&form)
        .map_err(http_error)?;
    let v: Value = resp
        .into_json()
        .map_err(|e| format!("授权响应解析失败: {e}"))?;
    v.get("access_token")
        .and_then(|t| t.as_str())
        .map(String::from)
        .ok_or_else(|| "授权响应缺少 access_token".into())
}

/// 流式下载到 dest（已含 .part 后缀的真实落盘路径），回调输出进度行。
/// 返回 (字节数, sha256 hex)。失败时删除半成品（红线：不留半截文件冒充安装）。
pub fn download_to_file(
    url: &str,
    dest: &Path,
    mut on_line: impl FnMut(String),
) -> Result<(u64, String), String> {
    use sha2::{Digest, Sha256};
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .build();
    let resp = agent.get(url).call().map_err(http_error)?;
    let total = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok());
    let mut reader = BufReader::new(resp.into_reader());
    let mut file = File::create(dest).map_err(|e| format!("创建 {} 失败: {e}", dest.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    let mut written: u64 = 0;
    let next_report = if let Some(t) = total {
        t / 10
    } else {
        8 * 1024 * 1024
    };
    let mut next = next_report;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("下载读失败: {e}"));
        let n = match n {
            Ok(n) => n,
            Err(e) => {
                drop(file);
                let _ = std::fs::remove_file(dest);
                return Err(e);
            }
        };
        if n == 0 {
            break;
        }
        if file.write_all(&buf[..n]).is_err() {
            drop(file);
            let _ = std::fs::remove_file(dest);
            return Err(format!("下载写 {} 失败", dest.display()));
        }
        hasher.update(&buf[..n]);
        written += n as u64;
        if written >= next {
            on_line(format!("已下载 {} bytes…", written));
            next = written + next_report.max(1);
        }
    }
    on_line(format!("下载完成: {} bytes", written));
    Ok((written, format!("{:x}", hasher.finalize())))
}

/// 从下载 URL 提取文件名（去 query）；失败回退 fallback。
pub fn filename_from_url(url: &str, fallback: &str) -> String {
    url.split('?')
        .next()
        .and_then(|p| p.rsplit('/').next())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| fallback.into())
}
