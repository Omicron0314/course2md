# CLI installation and usage

[Wiki](Home.md) · [GUI](GUI.md) · [CLI](CLI.md) · [中文](CLI.zh.md)

This page covers the command-line application. For the desktop window, see the [GUI guide](GUI.md). The Homebrew formula, `course2md-bin`, and install.sh install only the CLI.

## Quick Start

> Make sure you completed the [Installation](CLI.md#installation) section first.

Simply provide an online video URL or a path to a local video file. When the run finishes, the illustrated notes (`course.md` / `course.html`) appear under `./out/<platform>/<title>/<id>/`:

```bash
# Process a Bilibili video
course2md https://www.bilibili.com/video/BV1pb8o6yE8f

# Process a YouTube video
course2md https://youtu.be/dQw4w9WgXcQ

# Process a local lecture or meeting recording
course2md ./lecture.mp4
```

> **First Run Note**: The very first run (`course2md <URL or file>` with no config file yet and no `--provider` flag, in an interactive terminal) launches a setup wizard for speech recognition:
> - **Local or cloud first**: **Local recognition** (recommended — offline, free, private; model download required) or **Cloud API** (no model download; needs an OpenAI-compatible endpoint API key, pay-as-you-go).
> - **Local backends, matched to your machine**: the recommended option is listed first — `coreml` (Apple native) on macOS Apple Silicon, `gpu` when `llama-server` is installed, `npu` on Intel NPU machines, and `cpu` when `llama-server` is installed.
> - **`gpu` / `cpu` chosen**: reuses complete cached models; otherwise confirms whether to download the ~2.4 GB model now, switch to the cloud API instead, or exit and download later via `course2md models download`.
> - **`coreml` chosen (macOS)**: the model is downloaded on first transcription (~1–2.3 GB to `~/Library/Caches/qwen3-speech/`); an interactive prompt lets you pick **qwen3-1.7b** (default — Qwen3-ASR 1.7B MLX, most accurate) / **qwen3-0.6b** (CoreML on ANE, power-sipping, ~1 GB) / **whisper** (large-v3-turbo, multilingual).
> - **Cloud API chosen**: prompts for base URL (defaults to OpenRouter), API key (hidden input; may be left empty if `COURSE2MD_ASR_API_KEY` is already set), and model name.
> - The choice is saved to `~/.config/course2md/config.toml` — override it any time with `--provider` or by editing the file directly. Non-interactive environments (CI, pipes) skip the wizard and use the platform defaults.
> - **Slow / blocked network?** Set a HuggingFace mirror first: `export HF_ENDPOINT=https://hf-mirror.com` (download errors print this hint too) — or skip local models entirely with `course2md <URL> --provider api`.
> - Tip: pre-download the offline model any time with `course2md models download`.

---


## Installation

`course2md` relies on the following multimedia tools:
- `ffmpeg` & `ffprobe` (Audio/video extraction and slide sampling)
- `yt-dlp` (Online video parsing and downloading; only needed for online URLs)
- `llama-server` (Provided by `llama.cpp`; only needed for local `gpu` / `cpu` backends, not required for macOS `coreml` or cloud `api` mode)

Once installed, just run `course2md <URL or file>` — the first run interactively guides you through picking a speech recognition backend (see [First Run Note](CLI.md#quick-start)).

---

### macOS

> Requires **macOS 15 (Sequoia) or later** on Apple Silicon (the CoreML backend depends on the ANE runtime shipped with macOS 15+; Intel Macs fall back to the `gpu`/`cpu` backends).

**Homebrew (recommended)** — dependencies, the Developer-ID-signed binary and the CoreML `mlx.metallib` are all handled for you:

```bash
brew install mizorewww/tap/course2md
```

<details>
<summary>Alternative: install.sh</summary>

```bash
brew install ffmpeg yt-dlp   # llama.cpp only needed for the gpu/cpu fallback backend
curl -fsSL https://raw.githubusercontent.com/mizorewww/course2md/main/install.sh | bash
```
</details>


---

### Arch Linux / CachyOS

Available on the **AUR** with automated dependency resolution:

```bash
# Install via AUR helper (first-class citizen)
yay -S course2md-bin
# or using paru:
# paru -S course2md-bin
```

<details>
<summary>Manual installation</summary>

```bash
# 1. Install dependencies
sudo pacman -S ffmpeg yt-dlp llama-cpp
# GPU ASR also needs a backend and GPU driver (Intel example)
sudo pacman -S ggml-vulkan vulkan-intel
llama-server --list-devices

# 2. Install course2md
curl -fsSL https://raw.githubusercontent.com/mizorewww/course2md/main/install.sh | bash
```
</details>

---

### Debian / Ubuntu

```bash
# 1. Install base dependencies and build tools
sudo apt update
sudo apt install -y ffmpeg yt-dlp git cmake build-essential

# 2. Build and install llama-server
git clone https://github.com/ggml-org/llama.cpp.git
cmake -S llama.cpp -B llama.cpp/build -DLLAMA_CURL=OFF
cmake --build llama.cpp/build --config Release -j
sudo install -m755 llama.cpp/build/bin/llama-server /usr/local/bin/llama-server

# 3. Install course2md
curl -fsSL https://raw.githubusercontent.com/mizorewww/course2md/main/install.sh | bash
```


---

### Windows

Install dependencies via `winget` in **PowerShell**:

```powershell
winget install --id Gyan.FFmpeg -e
winget install --id yt-dlp.yt-dlp -e
winget install --id ggml.llamacpp -e
```

> Alternatively, install via Scoop: `scoop install ffmpeg yt-dlp` (for local `gpu`/`cpu` ASR you additionally need `llama-server.exe` from llama.cpp releases on your `PATH`).

**Install course2md**:
1. Download `course2md-windows-x86_64.exe` from [Releases](https://github.com/mizorewww/course2md/releases).
2. Rename to `course2md.exe` and place it in a directory listed in your `PATH`.


---

### Building from Source

Requires the stable Rust toolchain:

```bash
git clone https://github.com/mizorewww/course2md.git
cd course2md

# Standard install
cargo install --path .

# Or build release binary only
cargo build --release
```

- **macOS Apple Silicon Note**: Building native CoreML support requires Xcode 16+ (Swift 6 toolchain). `build.rs` compiles the Swift package and copies `mlx.metallib` to the target directory. If you do not need native CoreML support, skip it via: `COURSE2MD_NO_APPLE=1 cargo build --release`.
- **Other Platforms**: Linux, Windows, and x86_64 macOS builds automatically skip Apple-native components.

---


## CLI Options

| Option | Description | Default |
| :--- | :--- | :--- |
| `-o, --out <DIR>` | Output root directory | `out` |
| `--transcript-source <auto/subtitle/asr>` | Transcript source: `auto` = platform subtitles first (manual > auto-caption), fall back to local ASR; `subtitle` = fail if none; `asr` = skip subtitles | `auto` |
| `--provider <coreml/gpu/cpu/api/npu>` | ASR backend: `coreml` (macOS arm64), `gpu` (non-Mac), `cpu`, or `api` (cloud STT) | Platform default |
| `--gpu-layers <0-99>` | GPU offload layers for `llama-server` (`-ngl`); try lowering it if AMD/ROCm iGPUs hang | `99` |
| `--mmproj-offload` / `--no-mmproj-offload` | Offload the multimodal projector to GPU or keep it on CPU | Offload |
| `--asr-model <qwen3-1.7b/qwen3-0.6b/whisper>` | CoreML ASR model variant: `qwen3-1.7b` (default, MLX on GPU), `qwen3-0.6b` (CoreML on ANE, low power), or `whisper` (large-v3-turbo) | `qwen3-1.7b` |
| `--asr-api-base-url <URL>` | Cloud STT base URL (OpenAI-compatible) | `https://openrouter.ai/api/v1` |
| `--asr-api-key <KEY>` | Cloud STT API Key (or set `COURSE2MD_ASR_API_KEY` env) | Config / Env |
| `--asr-api-model <MODEL>` | Cloud STT model slug (e.g. `qwen/qwen3-asr-flash-2026-02-10`) | `qwen/qwen3-asr-flash-2026-02-10` |
| `--similarity <0~1>` | SSIM similarity threshold; **higher = more sensitive = more slides captured** | `0.85` |
| `--sample-interval <SEC>` | Frame sampling check interval in seconds | `1.0` |
| `--cooldown <SEC>` | Minimum seconds between two consecutive slide captures | `10.0` |
| `--roi <x1,y1-x2,y2>` | Region of interest for slide comparison (e.g. `40%,0%-100%,100%`) | Full frame |
| `--formats <FORMATS>` | Comma-separated output formats: `md,html,json` | `md,html` |
| `--threads <N>` | Number of ASR worker threads (for local `gpu`/`cpu`) | `4` |
| `--max-speech <SEC>` | Maximum speech segment duration in seconds | `20.0` |
| `--keep-video` | Preserve downloaded/extracted `media.mp4` | Disabled |
| `--no-download` | Skip downloading (when `media.mp4` exists in directory) | Disabled |
| `--llm` | Force enable LLM subtitle polishing for this run | Disabled |
| `--no-llm` | Force disable LLM subtitle polishing for this run | Disabled |
| `--llm-vision` | Vision-assisted polish: attach the section slide to correct technical terms (multimodal model required) | Disabled |
| `--no-llm-vision` | Disable vision-assisted polish for this run | Disabled |
| `--no-llm-hint` | Suppress post-run LLM suggestion hint | Disabled |
| `--resume` | Resume unfinished ASR chunks from the output dir | Disabled |
| `--no-resume` | Discard existing checkpoints and redo everything | Disabled |
| `-v, --verbose` | Add diagnostic logs (`-v`) or debug details (`-vv`) | Stages and warnings |
| `-q, --quiet` | Quiet mode (errors only) | Disabled |

Display full help:

```bash
course2md --help
```

---
