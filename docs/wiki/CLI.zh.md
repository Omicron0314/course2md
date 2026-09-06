# CLI 安装与使用

[Wiki](Home.zh.md) · [GUI](GUI.zh.md) · [CLI](CLI.zh.md) · [English](CLI.md)

本页仅介绍命令行版本。使用桌面窗口请看 [GUI 指南](GUI.zh.md)。Homebrew Formula、`course2md-bin` 和 install.sh 都只安装 CLI。

## 快速上手

> 请先完成[安装指南](CLI.zh.md#安装指南)。

传入在线视频 URL 或本地视频文件路径即可开始转换。完成后，图文笔记（`course.md` / `course.html`）保存在 `./out/<平台>/<标题>/<编号>/` 目录：

```bash
# 解析 B 站视频
course2md https://www.bilibili.com/video/BV1pb8o6yE8f

# 解析 YouTube 视频
course2md https://youtu.be/dQw4w9WgXcQ

# 解析本地课件/会议录屏
course2md ./lecture.mp4
```

> **首次运行说明**：首次运行（配置文件尚不存在、未传 `--provider`、且处于交互式终端）会进入配置向导，引导设置语音转写方式：
> - **先选本地还是云端**：**本地识别**（推荐——离线、免费、隐私；首次需下载模型）或**云端 API**（免下载模型；需 OpenAI 兼容端点 API key，按量计费）。
> - **本地后端按本机能力列出**：推荐项置顶——macOS Apple Silicon 上为 `coreml`（Apple 原生），装有 `llama-server` 时可选 `gpu`，Intel NPU 机器可选 `npu`，装有 `llama-server` 时也可选 `cpu`。
> - **选 `gpu` / `cpu`**：复用已完整下载的模型；缺少模型时确认是否现在下载约 2.4GB，也可改为云端 API，或退出后稍后运行 `course2md models download` 手动下载。
> - **选 `coreml`（macOS）**：模型在首次识别时才下载（约 1~2.3GB，保存到 `~/Library/Caches/qwen3-speech/`），届时可交互选择 **qwen3-1.7b**（默认——Qwen3-ASR 1.7B MLX，中文/中英混合最准）/ **qwen3-0.6b**（CoreML 走 ANE，省电低功耗，约 1GB）/ **whisper**（large-v3-turbo，多语种）。
> - **选云端 API**：依次引导填写 base URL（默认 OpenRouter）、API Key（输入隐藏；已设置 `COURSE2MD_ASR_API_KEY` 时可留空）与模型名。
> - 选择会写入 `~/.config/course2md/config.toml`——以后可用 `--provider` 临时切换，或直接编辑配置文件。非交互环境（CI、管道）不触发向导，走平台默认。
> - **网络受限？** 先设 HuggingFace 镜像：`export HF_ENDPOINT=https://hf-mirror.com`（下载失败时错误信息也会提示）；或直接 `course2md <URL> --provider api` 免本地模型试用。
> - 提示：随时可用 `course2md models download` 预先下载离线识别模型。

---


## 安装指南

运行 `course2md` 依赖以下基础多媒体工具：
- `ffmpeg` & `ffprobe`（音视频抽取与画面采样）
- `yt-dlp`（在线视频解析与下载；仅处理在线链接时需要）
- `llama-server`（由 `llama.cpp` 提供；仅在本地 `gpu` / `cpu` 识别后端下需要，macOS `coreml` 与云端 `api` 模式无需安装）

装完直接运行 `course2md <URL或文件>` 即可——首次运行会引导配置语音识别后端（见上文[首次运行说明](CLI.zh.md#快速上手)）。

---

### macOS

> 要求 **macOS 15 (Sequoia) 及以上**（Apple Silicon 的 CoreML 后端依赖 macOS 15+ 的 ANE 运行时；Intel Mac 自动回落 `gpu`/`cpu` 后端）。

**Homebrew（推荐）**——依赖、Developer ID 签名的二进制和 CoreML 所需的 `mlx.metallib` 一次装齐：

```bash
brew install mizorewww/tap/course2md
```

<details>
<summary>备选：install.sh 脚本</summary>

```bash
brew install ffmpeg yt-dlp   # llama.cpp 仅在 gpu/cpu 兜底后端时需要
curl -fsSL https://raw.githubusercontent.com/mizorewww/course2md/main/install.sh | bash
```
</details>


---

### Arch Linux / CachyOS

推荐直接通过 **AUR** 安装，自动配置所有依赖与软链接：

```bash
# 通过 AUR 助手安装（一等公民支持）
yay -S course2md-bin
# 或使用 paru:
# paru -S course2md-bin
```

<details>
<summary>手动安装方式</summary>

```bash
# 1. 安装系统依赖
sudo pacman -S ffmpeg yt-dlp llama-cpp
# GPU 识别还需后端和显卡驱动（Intel 示例）
sudo pacman -S ggml-vulkan vulkan-intel
llama-server --list-devices

# 2. 安装 course2md
curl -fsSL https://raw.githubusercontent.com/mizorewww/course2md/main/install.sh | bash
```
</details>

---

### Debian / Ubuntu

```bash
# 1. 安装基础依赖与编译工具
sudo apt update
sudo apt install -y ffmpeg yt-dlp git cmake build-essential

# 2. 编译并安装 llama-server
git clone https://github.com/ggml-org/llama.cpp.git
cmake -S llama.cpp -B llama.cpp/build -DLLAMA_CURL=OFF
cmake --build llama.cpp/build --config Release -j
sudo install -m755 llama.cpp/build/bin/llama-server /usr/local/bin/llama-server

# 3. 安装 course2md
curl -fsSL https://raw.githubusercontent.com/mizorewww/course2md/main/install.sh | bash
```


---

### Windows

在 **PowerShell** 中使用 `winget` 一键安装依赖：

```powershell
winget install --id Gyan.FFmpeg -e
winget install --id yt-dlp.yt-dlp -e
winget install --id ggml.llamacpp -e
```

> 也可以通过 Scoop (`scoop install ffmpeg yt-dlp`) 或 Chocolatey 安装。请确保 `ffmpeg`、`ffprobe`、`yt-dlp`、`llama-server.exe` 均已加入系统 `PATH`。

**安装 course2md**：
1. 前往 [Releases](https://github.com/mizorewww/course2md/releases) 下载 `course2md-windows-x86_64.exe`。
2. 重命名为 `course2md.exe` 并将其移动至已加入系统 `PATH` 的目录中。


---

### 从源码构建

需要安装 Rust 稳定版工具链：

```bash
git clone https://github.com/mizorewww/course2md.git
cd course2md

# 标准构建与安装
cargo install --path .

# 或仅编译 Release 二进制文件
cargo build --release
```

- **macOS Apple Silicon 说明**：构建原生 CoreML 支持需要系统安装 Xcode 16+（包含 Swift 6 工具链）。`build.rs` 会自动编译 Swift 模块并将 `mlx.metallib` 复制到 target 目录。如果不需要 CoreML 模块，可通过环境变量跳过：`COURSE2MD_NO_APPLE=1 cargo build --release`。
- **其他平台**：Linux、Windows 以及 x86_64 macOS 构建时会自动跳过 Apple 原生模块。

---


## 常用参数

| 参数 | 说明 | 默认值 |
| :--- | :--- | :--- |
| `-o, --out <目录>` | 指定输出根目录 | `out` |
| `--transcript-source <auto/subtitle/asr>` | 转写来源：`auto` = 平台字幕优先（人工>自动），无字幕再走本地 ASR；`subtitle` = 强制字幕（无则报错）；`asr` = 跳过字幕直接识别 | `auto` |
| `--provider <coreml/gpu/cpu/api/npu>` | 识别后端：`coreml`（macOS 默认）、`gpu`（非 Mac 默认）、`cpu`、`api`（云端 STT） | 视平台而定 |
| `--gpu-layers <0-99>` | `llama-server` 的 GPU 卸载层数（`-ngl`）；AMD/ROCm 核显遇 hang 时可尝试调低 | `99` |
| `--mmproj-offload` / `--no-mmproj-offload` | 多模态 projector 是否卸载到 GPU | 卸载 |
| `--asr-model <qwen3-1.7b/qwen3-0.6b/whisper>` | CoreML 识别模型变体：`qwen3-1.7b`（默认，MLX 走 GPU）、`qwen3-0.6b`（CoreML 走 ANE，省电）或 `whisper`（large-v3-turbo） | `qwen3-1.7b` |
| `--asr-api-base-url <URL>` | 云端 STT base URL（OpenAI 兼容） | `https://openrouter.ai/api/v1` |
| `--asr-api-key <KEY>` | 云端 STT API Key（亦可设置 `COURSE2MD_ASR_API_KEY` 环境变量） | 配置文件 / 环境变量 |
| `--asr-api-model <模型名>` | 云端 STT 模型名称（如 `qwen/qwen3-asr-flash-2026-02-10`） | `qwen/qwen3-asr-flash-2026-02-10` |
| `--similarity <0~1>` | SSIM 画面相似度阈值；**数值越高越敏感、截图越多** | `0.85` |
| `--sample-interval <秒>` | 画面采样检查间隔（秒） | `1.0` |
| `--cooldown <秒>` | 连续两张截图之间的最短间隔时间（秒） | `10.0` |
| `--roi <x1,y1-x2,y2>` | 只比较画面指定区域（如 `40%,0%-100%,100%`） | 全屏 |
| `--formats <格式>` | 输出格式，逗号分隔，可选 `md,html,json` | `md,html` |
| `--threads <数量>` | ASR 识别线程数（供本地 `gpu`/`cpu` 后端使用） | `4` |
| `--max-speech <秒>` | 单段语音最长切分秒数 | `20.0` |
| `--keep-video` | 保留下载或提取的原始 `media.mp4` 文件 | 关闭 |
| `--no-download` | 跳过下载（目录中已有 `media.mp4` 时） | 关闭 |
| `--llm` | 本次运行强制启用 LLM 字幕润色 | 关闭 |
| `--no-llm` | 本次运行强制禁用 LLM 字幕润色 | 关闭 |
| `--llm-vision` | 视觉润色：请求附对应幻灯片截图，辅助纠正技术词汇（需多模态模型） | 关闭 |
| `--no-llm-vision` | 本次运行关闭视觉润色 | 关闭 |
| `--no-llm-hint` | 本次运行关闭任务结束时的 LLM 开启提示 | 关闭 |
| `--resume` | 从输出目录续跑未完成的 ASR chunk | 关闭 |
| `--no-resume` | 丢弃既有进度，全部重算 | 关闭 |
| `-v, --verbose` | `-v` 显示诊断日志，`-vv` 显示调试细节 | 默认阶段和警告 |
| `-q, --quiet` | 静默模式，只显示错误 | 关闭 |

查看完整参数与子命令列表：

```bash
course2md --help
```

---
