// src/main.rs

mod api;
mod args;
mod lrc;
mod process;
mod utils;

use anyhow::{anyhow, Result};
use clap::Parser;
use std::io;
use std::path::Path;

fn main() -> Result<()> {
    let args = args::Cli::parse();

    validate_args(&args)?;

    if args.list {
        let api_url = args.api.as_ref().unwrap();
        let client = api::ApiClient::new(api_url.clone())?;
        let voices = client.list_voices()?;
        
        println!("可用的声音列表：");
        for voice in voices {
            println!("{}", "=".repeat(50));
            println!("ID: {},", voice.id);
            println!("名称: {},", voice.name);
            println!("性别: {},", voice.gender);
            println!("语言: {},", voice.locale);
            println!("类型: {}", voice.voice_type);
        }
        println!("{}", "=".repeat(50));

    } else if let Some(file_path) = &args.file {
        // --- 单文件处理逻辑也采用新的预扫描流程 ---
        let api_url = args.api.as_ref().unwrap();
        let client = api::ApiClient::new(api_url.clone())?;
        let blacklist = match &args.blacklist {
            Some(source) => Some(utils::load_blacklist(source)?),
            None => None,
        };

        // 对单个文件进行预扫描
        let files_to_check = vec![file_path.clone()];
        let files_to_convert = utils::pre_scan_for_encoding_issues(&files_to_check)?;

        if let Some(info) = files_to_convert.first() {
            println!("警告：文件 {:?} 的编码可能不是 UTF-8。", info.path);
            println!("检测为: {} (置信度: {:.1}%)", info.encoding, info.confidence * 100.0);
            println!("是否要将其转换为 UTF-8 编码？(此操作将覆盖原文件) (y/N)");
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                utils::convert_files_to_utf8(&files_to_convert)?;
            } else {
                return Err(anyhow!("用户取消了编码转换，终止操作。"));
            }
        }
        
        process::process_file(file_path, &args, &client, &blacklist)?;

    } else if let Some(dir_path) = &args.dir {
        let api_url = args.api.as_ref().unwrap();
        let client = api::ApiClient::new(api_url.clone())?;
        let blacklist = match &args.blacklist {
            Some(source) => Some(utils::load_blacklist(source)?),
            None => None,
        };
        process::process_directory(dir_path, &args, &client, &blacklist)?;

    } else {
        println!("错误：没有指定操作 (使用 -h 或 --help 获取帮助)");
    }

    Ok(())
}

fn validate_args(args: &args::Cli) -> Result<()> {
    let has_file_op = args.file.is_some() || args.dir.is_some();

    if args.list {
        if has_file_op || args.voice.is_some() || args.volume.is_some() || args.speed.is_some() || args.pitch.is_some() || args.sub.is_some() || args.blacklist.is_some() {
            return Err(anyhow!("使用 --list 时，只允许提供 --api 参数。"));
        }
        if args.api.is_none() {
            return Err(anyhow!("使用 --list 时，必须提供 --api 参数。"));
        }
    } else if has_file_op {
        if args.api.is_none() {
            return Err(anyhow!("使用 --file 或 --dir 时，必须提供 --api 参数。"));
        }
        if let Some(file) = &args.file {
            if !Path::new(file).exists() { return Err(anyhow!("文件不存在: {:?}", file)); }
        }
        if let Some(file) = &args.file && !Path::new(file).exists() { 
            return Err(anyhow!("文件不存在: {:?}", file)); 
        }
    }
    
    Ok(())
}
