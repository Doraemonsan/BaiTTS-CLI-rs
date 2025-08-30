// src/process.rs

use crate::api::ApiClient;
use crate::args::Cli;
use crate::lrc;
use crate::utils;
use anyhow::{Context, Result, anyhow};
use hound::{WavReader, WavWriter};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::Builder;

pub fn process_file(
    file_path: &Path,
    args: &Cli,
    client: &ApiClient,
    blacklist: &Option<Regex>,
) -> Result<()> {
    println!("正在处理文件: {:?}", file_path);

    let content =
        fs::read_to_string(file_path).context(format!("无法读取文件内容: {:?}", file_path))?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    if lines.is_empty() {
        println!("文件为空，跳过处理: {:?}", file_path);
        return Ok(());
    }

    //  在文件末添加一个静音 1000ms 标签
    if let Some(last_line) = lines.last_mut() {
        last_line.push_str("[[PAUSE:1000]]");
    }

    let temp_dir = Builder::new().prefix("baitts-").tempdir()?;
    let mut audio_chunks = Vec::new();
    let mut durations = Vec::new();
    let mut original_lines_for_lrc = Vec::new();

    let pb = ProgressBar::new(lines.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );

    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            pb.inc(1);
            continue;
        }

        let text_to_send = if let Some(bl_regex) = blacklist {
            utils::apply_blacklist(line, bl_regex)
        } else {
            line.clone()
        };

        let audio_data = client.generate_speech(
            &text_to_send,
            &args.voice,
            &args.volume,
            &args.speed,
            &args.pitch,
        )?;

        let chunk_path = temp_dir.path().join(format!("{:05}.wav", i));
        fs::write(&chunk_path, &audio_data)?;

        let reader = WavReader::new(BufReader::new(File::open(&chunk_path)?))?;
        let duration_ms = (reader.len() as f64 / reader.spec().sample_rate as f64) * 1000.0;
        durations.push(Duration::from_millis(duration_ms as u64));

        audio_chunks.push(chunk_path);
        if args.sub.is_some() {
            original_lines_for_lrc.push(line.clone());
        }
        pb.inc(1);
    }
    pb.finish_with_message("下载完成");

    let output_filename = file_path.file_stem().unwrap().to_str().unwrap();
    let output_path = args.out.join(format!("{}.wav", output_filename));

    fs::create_dir_all(&args.out).context("创建输出目录失败")?;

    combine_wav_files(&audio_chunks, &output_path)?;
    println!("成功合成音频文件: {:?}", output_path);

    if let Some(chars_per_line) = args.sub {
        lrc::generate_lrc(
            &output_path,
            &original_lines_for_lrc,
            &durations,
            chars_per_line,
        )?;
    }

    Ok(())
}

pub fn process_directory(
    dir_path: &Path,
    args: &Cli,
    client: &ApiClient,
    blacklist: &Option<Regex>,
) -> Result<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir_path)?
        .filter_map(|res| res.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "txt"))
        .collect();

    entries.sort();

    if entries.is_empty() {
        println!("在目录 {:?} 中没有找到 .txt 文件。", dir_path);
        return Ok(());
    }

    println!("找到 {} 个 .txt 文件，准备处理...", entries.len());

    // 预扫描和转换逻辑
    println!("正在扫描文件编码...");
    let files_to_convert = utils::pre_scan_for_encoding_issues(&entries)?;

    if !files_to_convert.is_empty() {
        println!("\n警告：检测到以下文件可能不是 UTF-8 编码：");
        for info in &files_to_convert {
            println!(
                "  - {:?} (检测为: {}, 置信度: {:.1}%)",
                info.path.file_name().unwrap_or_default(),
                info.encoding,
                info.confidence * 100.0
            );
        }
        println!("\n程序需要 UTF-8 编码才能正确处理文本。");
        println!("是否要将以上所有文件转换为 UTF-8 编码？");
        println!("注意：此操作将覆盖原文件！(y/N)");

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == "y" {
            utils::convert_files_to_utf8(&files_to_convert)?;
            println!("\n所有文件已转换完毕，开始执行 TTS 任务。");
        } else {
            return Err(anyhow!("用户取消了编码转换，终止操作。"));
        }
    } else {
        println!("所有文件编码兼容，准备开始 TTS 任务。");
    }
    println!("--------------------------------------------------");

    for entry in entries {
        process_file(&entry, args, client, blacklist)?;
        println!("--------------------------------------------------");
    }

    println!("所有文件处理完毕。");
    Ok(())
}

fn combine_wav_files(files: &[PathBuf], output_path: &Path) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }

    let reader = WavReader::open(&files[0])?;
    let spec = reader.spec();

    let mut writer = WavWriter::create(output_path, spec)?;

    for file in files {
        let mut reader = WavReader::open(file)?;
        if reader.spec() != spec {
            eprintln!(
                "警告: 文件 {:?} 的 WAV 格式与第一个文件不同，跳过合并。",
                file
            );
            continue;
        }
        let samples = reader.samples::<i16>().collect::<Result<Vec<_>, _>>()?;
        for sample in samples {
            writer.write_sample(sample)?;
        }
    }

    writer.finalize()?;
    Ok(())
}
