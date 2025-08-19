// src/args.rs

use clap::Parser;
use std::path::PathBuf;

/// BaiTTS-CLI-rs: 基于 MulitTTS API 的 TXT 转有声书命令行工具
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// [操作模式] 列出当前 API 所有可用的声音
    #[arg(short, long, group = "mode")]
    pub list: bool,

    /// [操作模式] 指定要处理的单个 .txt 文件
    #[arg(short, long, value_name = "FILE_PATH", group = "mode")]
    pub file: Option<PathBuf>,

    /// [操作模式] 指定要处理的包含多个 .txt 文件的目录
    #[arg(short, long, value_name = "DIR_PATH", group = "mode")]
    pub dir: Option<PathBuf>,

    /// [必需] MulitTTS API 的基础 URL [示例： http://127.0.0.1:8774]
    #[arg(long)]
    pub api: Option<String>,

    /// [可选] 指定输出目录
    #[arg(short, long, value_name = "OUTPUT_DIR", default_value = "output")]
    pub out: PathBuf,

    /// [可选] 指定要使用的声音 ID
    #[arg(long)]
    pub voice: Option<String>,

    /// [可选] 指定音量 (0-100)
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub volume: Option<u8>,

    /// [可选] 指定语速 (0-100)
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub speed: Option<u8>,

    /// [可选] 指定音高 (0-100)
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub pitch: Option<u8>,

    /// [可选] 生成 LRC 歌词文件，并设置每行最大字符数
    #[arg(short, long, value_name = "CHARS_PER_LINE", num_args(0..=1), default_missing_value="15", value_parser = parse_sub_range)]
    pub sub: Option<usize>,

    /// [可选] 指定黑名单词库的来源 (本地路径或 URL)
    #[arg(short = 'b', long, value_name = "SOURCE")]
    pub blacklist: Option<String>,
}

// 验证函数
fn parse_sub_range(s: &str) -> Result<usize, String> {
    let value: usize = s
        .parse()
        .map_err(|_| format!("'{}' 不是一个有效的数字", s))?;
    if (10..=100).contains(&value) {
        Ok(value)
    } else {
        Err(format!("LRC 歌词字符数必须在 10 到 100 之间"))
    }
}
