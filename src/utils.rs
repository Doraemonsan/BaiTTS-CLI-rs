// src/utils.rs

use anyhow::{anyhow, Context, Result};
use regex::{Regex, RegexBuilder};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use once_cell::sync::Lazy;

pub static LRC_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[\[.*?\]\]").unwrap()
});

pub fn load_blacklist(source: &str) -> Result<Regex> {
    let content = if source.starts_with("http://") || source.starts_with("https://") {
        reqwest::blocking::get(source)
            .context("无法获取远程黑名单文件")?
            .text()
            .context("无法读取远程黑名单内容")?
    } else if Path::new(source).exists() {
        fs::read_to_string(source).context("无法读取本地黑名单文件")?
    } else {
        source.to_string()
    };

    let patterns: Vec<String> = content
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("({})", s))
        .collect();

    if patterns.is_empty() {
        return Err(anyhow!("黑名单源为空或无效"));
    }

    let combined_pattern = patterns.join("|");
    RegexBuilder::new(&combined_pattern)
        .case_insensitive(true)
        .build()
        .context("编译黑名单正则表达式失败")
}

pub fn apply_blacklist(text: &str, blacklist_regex: &Regex) -> String {
    blacklist_regex
        .replace_all(text, "[[$0]]")
        .to_string()
}

/// 存储需要进行编码转换的文件信息
#[derive(Debug)]
pub struct EncodingInfo {
    pub path: PathBuf,
    pub encoding: String,
    pub confidence: f32,
}

/// 预扫描文件列表，找出所有非 UTF-8 编码的文件。
/// 优化效率，每个文件只读取前 8KB。
pub fn pre_scan_for_encoding_issues(paths: &[PathBuf]) -> Result<Vec<EncodingInfo>> {
    const READ_BUFFER_SIZE: u64 = 8192;
    const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
    const UTF8_CONFIDENCE_THRESHOLD: f32 = 0.95;

    let mut issues = Vec::new();

    for path in paths {
        let file = File::open(path)
            .context(format!("预扫描时无法打开文件: {:?}", path))?;

        // 读取文件的前 8KB
        let mut buffer = Vec::new();
        file.take(READ_BUFFER_SIZE).read_to_end(&mut buffer)?;

        if buffer.is_empty() {
            continue; // 跳过空文件
        }

        // 优先检查 UTF-8 BOM
        if buffer.starts_with(UTF8_BOM) {
            continue;
        }

        let (encoding, confidence, ..) = chardet::detect(&buffer);

        // 如果不是高置信度的 UTF-8，则记录下来
        if !(encoding.eq_ignore_ascii_case("UTF-8") && confidence > UTF8_CONFIDENCE_THRESHOLD) {
            issues.push(EncodingInfo {
                path: path.clone(),
                encoding,
                confidence,
            });
        }
    }

    Ok(issues)
}

/// 转换指定文件列表到 UTF-8 编码，此操作会覆盖原文件。
pub fn convert_files_to_utf8(files_to_convert: &[EncodingInfo]) -> Result<()> {
    for info in files_to_convert {
        println!("正在转换文件: {:?}...", info.path);
        
        let bytes = fs::read(&info.path)
            .context(format!("无法读取文件以进行转换: {:?}", info.path))?;

        let (decoded, _, had_errors) = encoding_rs::Encoding::for_label(info.encoding.as_bytes())
            .unwrap_or(encoding_rs::WINDOWS_1252)
            .decode(&bytes);

        if had_errors {
            eprintln!("警告: 文件 {:?} 在转换为 UTF-8 时可能存在解码错误。", info.path);
        }

        fs::write(&info.path, decoded.as_bytes())
            .context(format!("无法将 UTF-8 内容写入文件: {:?}", info.path))?;
        
        println!("文件 {:?} 已成功转换为 UTF-8 编码。", info.path);
    }
    Ok(())
}
