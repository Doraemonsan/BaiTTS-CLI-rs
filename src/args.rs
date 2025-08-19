// src/args.rs

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// [操作模式] 列出所有可用的声音
    #[arg(short, long, group = "mode")]
    pub list: bool,

    /// [操作模式] 指定要处理的单个 .txt 文件
    #[arg(short, long, value_name = "FILE_PATH", group = "mode")]
    pub file: Option<PathBuf>,

    /// [操作模式] 指定要处理的包含多个 .txt 文件的目录
    #[arg(short, long, value_name = "DIR_PATH", group = "mode")]
    pub dir: Option<PathBuf>,

    /// [必需] 文本转语音 API 的基础 URL
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

    /// [可选] 生成 LRC 歌词文件，并指定每行最大字符数
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
