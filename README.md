# BaiTTS-CLI-rs

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.html)
[![Rust](https://img.shields.io/badge/rust-1.89.0-orange.svg)](https://www.rust-lang.org/)

一个基于 [MultiTTS](https://t.me/MultiTTS) API 的命令行工具，用于将文本文档（.txt）转换为有声书音频（.wav），并可选择生成同步的 LRC 歌词文件。使用 Python 开发的同款工具  [BaiTTS-CLI (已停止维护)](https://github.com/Doraemonsan/BaiTTS-CLI) 

## ✨ 功能特性

- **文本转语音**: 将单个 `.txt` 文件或整个目录的 `.txt` 文件转换为 WAV 音频。
- **LRC 歌词**: 在生成音频的同时，可以创建同步的 `.lrc` 歌词文件。
- **参数可调**: 支持自定义声音、音量、语速和音高。
- **编码处理**: 自动检测非 UTF-8 编码的文件，并提示用户进行转换。
- **黑名单词汇**: 支持通过本地文件或 URL 加载黑名单词库，以跳过特定词语的发音。

## ⚙️ 安装
安装方法二选一即可，推荐直接使用预构建的二进制文件，如果预构建的二进制文件不能满足你的运行平台，则建议自行编译安装。

### 1. 使用预构建二进制文件
根据你的系统下载最新的预构建二进制文件 [https://github.com/Doraemonsan/BaiTTS-CLI-rs/releases](https://github.com/Doraemonsan/BaiTTS-CLI-rs/releases) ，解压并赋予可执行权限直接执行，或者拷贝到你的系统路径 (如 `/usr/local/bin` ）下以全局使用

预构建的二进制文件现已支持:
  + Linux (glibc-x64, glibc-Arm64)
  + Windows (x64)
  + MacOS 15+ (Arm64)

### 2. 编译安装

首先，你需要安装 Rust 开发环境。推荐使用 `rustup` 进行安装。本项目在 `rustc 1.89.0` 版本下进行开发和测试，建议使用的 `rustc` 版本不低于本项目开发环境

```Shell
# 安装 rustup (如果尚未安装)
pacman -Sy rustup
# 安装最新的稳定版 rust 开发环境
rustup install stable
# 设置稳定版为默认环境
rustup default stable
```

使用源码进行构建

```Shell
# 1. 克隆本仓库
git clone https://github.com/Doraemonsan/BaiTTS-CLI-rs

# 2. 进入项目目录
cd BaiTTS-CLI-rs

# 3. 使用 Cargo 进行编译，如需交叉编译请自行安装对应平台工具链
# 使用 --target 来生成目标平台的二进制文件(如 --target x86_64-pc-windows-gnu)
cargo build --release

# 编译后的可执行文件位于 ./target/release/baitts-cli-rs
# 你可以将其复制到你的系统路径下（如 /usr/local/bin）以便全局使用
# sudo cp ./target/release/baitts-cli-rs /usr/local/bin
```

## 🚀 使用方法

**重要提示**: 所有操作都需要通过 `--api` 参数指定 `MultiTTS` 服务的 URL。

### 1. 查看可用的声音列表

```Shell
baitts-cli-rs --api http://127.0.0.1:8774 --list
```

### 2. 转换单个文本文件

```Shell
baitts-cli-rs --api http://127.0.0.1:8774 --file /path/to/your/book.txt
```

### 3. 批量转换目录下的所有文本文件

程序会自动查找并处理指定目录下的所有 `.txt` 文件。

```Shell
baitts-cli-rs --api http://127.0.0.1:8774 --dir /path/to/your/books/
```

### 4. 使用高级选项 (生成LRC、指定声音等)

```Shell
baitts-cli-rs \
  --api http://127.0.0.1:8774 \
  --file story.txt \
  --out ./audiobooks \
  --voice "zh-CN-XiaoxiaoNeural" \
  --speed 85 \
  --sub 25 \
  --blacklist ./my_blacklist.txt
```

- 此命令会将 `story.txt` 转换为音频。
- 输出文件保存在 `./audiobooks` 目录。
- 使用名为 `zh-CN-XiaoxiaoNeural` 的声音。
- 语速设置为 `85`。
- 生成 LRC 歌词，每行约 `25` 个字符。
- 使用本地的 `my_blacklist.txt` 文件的内容作为黑名单词汇。

## 📚 命令行参数

| 参数                | 缩写         | 描述                                                         | 默认值   |
| ------------------- | ------------ | ------------------------------------------------------------ | -------- |
| `--list`            | `-l`         | 列出当前 API 所有可用的声音。                                | -        |
| `--file <PATH>`     | `-f <PATH>`  | 指定要处理的单个 `.txt` 文件。                               | -        |
| `--dir <PATH>`      | `-d <PATH>`  | 指定要处理的包含多个 `.txt` 文件的目录。                     | -        |
| `--api <URL>`       |              | **[必需]** MultiTTS API 的基础 URL。                         | -        |
| `--out <DIR>`       | `-o <DIR>`   | 指定输出目录。                                               | `output` |
| `--voice <ID>`      |              | 指定要使用的声音 ID。                                        | API 默认 |
| `--volume <0-100>`  |              | 指定音量。                                                   | API 默认 |
| `--speed <0-100>`   |              | 指定语速。                                                   | API 默认 |
| `--pitch <0-100>`   |              | 指定音高。                                                   | API 默认 |
| `--sub [CHARS]`     | `-s [CHARS]` | 生成 LRC 歌词，并可选设置每行最大字符数 (10-100)。           | `15`     |
| `--blacklist <SRC>` | `-b <SRC>`   | 指定黑名单词库的来源 (本地路径或 URL)。多个字词使用管道符分割，支持正则，当输入为文件时，每行视为一个参数 | -        |
| `--help`            | `-h`         | 显示帮助信息。                                               | -        |
| `--version`         | `-V`         | 显示版本信息。                                               | -        |

## 📄 许可证

本项目采用 [GPLv3](https://www.gnu.org/licenses/gpl-3.0.html) 许可证。

## 问题反馈
如果您遇到任何问题，请通过 GitHub Issues 页面提交问题报告。
