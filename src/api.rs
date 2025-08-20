// src/api.rs

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct Voice {
    pub id: String,
    pub name: String,
    pub gender: String,
    pub locale: String,
    #[serde(rename = "type")]
    pub voice_type: String,
}

#[derive(Debug, Deserialize)]
struct VoicesCatalog {
    #[serde(flatten)]
    providers: HashMap<String, Vec<Voice>>,
}

#[derive(Debug, Deserialize)]
struct VoicesData {
    catalog: VoicesCatalog,
}

#[derive(Debug, Deserialize)]
struct VoicesResponse {
    success: bool,
    data: VoicesData,
}

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .context("无法创建 HTTP 客户端")?;
        Ok(Self { client, base_url })
    }

    fn send_request_with_retry<F>(&self, build_request: F) -> Result<reqwest::blocking::Response>
    where
        F: Fn() -> Result<reqwest::blocking::Request, reqwest::Error>,
    {
        let mut last_error = None;
        for attempt in 1..=3 {
            let request = build_request().context("构建请求失败")?;
            match self.client.execute(request) {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        last_error = Some(anyhow!(
                            "API 请求失败，状态码: {}",
                            response.status()
                        ));
                    }
                }
                Err(e) => {
                    last_error = Some(e.into());
                }
            }
            if attempt < 3 {
                println!("请求失败，将在 2 秒后重试... (尝试次数 {}/3)", attempt);
                std::thread::sleep(Duration::from_secs(2));
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow!("未知请求错误")))
            .context("API 请求在 3 次尝试后仍然失败")
    }

    pub fn list_voices(&self) -> Result<Vec<Voice>> {
        let url = format!("{}/voices", self.base_url);
        
        let response = self.send_request_with_retry(|| {
            self.client.get(&url).build()
        })?;

        let parsed_response = response
            .json::<VoicesResponse>()
            .context("解析声音列表 JSON 响应失败")?;

        if !parsed_response.success {
            return Err(anyhow!("API 返回 'success: false'"));
        }

        let all_voices = parsed_response
            .data
            .catalog
            .providers
            .into_values()
            .flatten()
            .collect();
        Ok(all_voices)
    }

    pub fn generate_speech(
        &self,
        text: &str,
        voice: &Option<String>,
        volume: &Option<u8>,
        speed: &Option<u8>,
        pitch: &Option<u8>,
    ) -> Result<Vec<u8>> {
        let mut url = url::Url::parse(&format!("{}/forward", self.base_url))
            .context("无法解析 /forward API 地址")?;
        
        url.query_pairs_mut().append_pair("text", text);
        if let Some(v) = voice { url.query_pairs_mut().append_pair("voice", v); }
        if let Some(v) = volume { url.query_pairs_mut().append_pair("volume", &v.to_string()); }
        if let Some(v) = speed { url.query_pairs_mut().append_pair("speed", &v.to_string()); }
        if let Some(v) = pitch { url.query_pairs_mut().append_pair("pitch", &v.to_string()); }

        let request_url = url.to_string();
        let response = self.send_request_with_retry(|| {
            self.client.get(&request_url).build()
        })?;

        let bytes = response.bytes().context("读取音频响应体失败")?.to_vec();
        Ok(bytes)
    }
}
