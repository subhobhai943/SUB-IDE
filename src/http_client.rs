use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub name: Option<String>,
    pub method: String, // GET, POST, PUT, DELETE, PATCH, etc.
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub elapsed_ms: f64,
}

pub struct HttpFileParser;

impl HttpFileParser {
    /// Parse a .http or .rest file content into a list of HttpRequests
    pub fn parse_file(content: &str) -> Vec<HttpRequest> {
        let mut requests = Vec::new();
        let mut current_name: Option<String> = None;
        let mut current_lines = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("###") {
                if !current_lines.is_empty() {
                    if let Some(req) = Self::parse_single_request(current_name.take(), &current_lines) {
                        requests.push(req);
                    }
                    current_lines.clear();
                }
                let name = trimmed.trim_start_matches('#').trim();
                if !name.is_empty() {
                    current_name = Some(name.to_string());
                }
            } else {
                current_lines.push(line.to_string());
            }
        }

        if !current_lines.is_empty() {
            if let Some(req) = Self::parse_single_request(current_name, &current_lines) {
                requests.push(req);
            }
        }

        requests
    }

    fn parse_single_request(name: Option<String>, lines: &[String]) -> Option<HttpRequest> {
        let mut iter = lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty() && !l.starts_with('#'));

        // First non-comment, non-empty line must be: METHOD URL (e.g. GET https://...)
        let request_line = iter.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_uppercase();
        let url = parts.next()?.to_string();

        let mut headers = HashMap::new();
        let mut body_lines = Vec::new();
        let mut reading_body = false;

        for line in lines {
            let trimmed = line.trim();
            if !reading_body {
                if trimmed.is_empty() {
                    // Empty line separates headers from body
                    reading_body = true;
                } else if trimmed.contains(':') && !trimmed.starts_with("http") && !trimmed.starts_with('#') {
                    if let Some((k, v)) = trimmed.split_once(':') {
                        headers.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
            } else {
                body_lines.push(line.as_str());
            }
        }

        let body = if body_lines.is_empty() {
            None
        } else {
            Some(body_lines.join("\n").trim().to_string())
        };

        Some(HttpRequest {
            name,
            method,
            url,
            headers,
            body,
        })
    }

    /// Execute an HTTP request using curl CLI (system dependency free)
    pub fn execute_request(req: &HttpRequest) -> Result<HttpResponse, String> {
        let start = std::time::Instant::now();
        let mut cmd = std::process::Command::new("curl");
        cmd.arg("-s").arg("-i").arg("-X").arg(&req.method);

        for (k, v) in &req.headers {
            cmd.arg("-H").arg(format!("{}: {}", k, v));
        }

        if let Some(ref body) = req.body {
            cmd.arg("-d").arg(body);
        }

        cmd.arg(&req.url);

        let output = cmd.output().map_err(|e| format!("curl execution failed: {}", e))?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let raw = String::from_utf8_lossy(&output.stdout);
        let (header_part, body_part) = if let Some((h, b)) = raw.split_once("\r\n\r\n") {
            (h.to_string(), b.to_string())
        } else if let Some((h, b)) = raw.split_once("\n\n") {
            (h.to_string(), b.to_string())
        } else {
            (String::new(), raw.to_string())
        };

        let mut status_code = 200;
        let mut status_text = "OK".to_string();
        let mut resp_headers = HashMap::new();

        for (idx, line) in header_part.lines().enumerate() {
            if idx == 0 && line.starts_with("HTTP/") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    status_code = parts[1].parse::<u16>().unwrap_or(200);
                    if parts.len() >= 3 {
                        status_text = parts[2..].join(" ");
                    }
                }
            } else if let Some((k, v)) = line.split_once(':') {
                resp_headers.insert(k.trim().to_string(), v.trim().to_string());
            }
        }

        Ok(HttpResponse {
            status_code,
            status_text,
            headers: resp_headers,
            body: body_part,
            elapsed_ms: elapsed,
        })
    }
}
