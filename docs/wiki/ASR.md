# Speech recognition and models

[Wiki](Home.md) · [GUI](GUI.md) · [CLI](CLI.md) · [中文](ASR.zh.md)

## ASR Backends

`course2md` provides multiple speech recognition backends via `--provider <backend>` or configuration:

| Backend (`--provider`) | Target & Default Policy | Architecture & Models | External Dependencies | Model Download & Cache Path | Highlights |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`coreml`** | **macOS Apple Silicon**<br>(Default for prebuilt arm64) | **Silero VAD v6.2.1 CoreML** (ANE)<br>+ **Qwen3-ASR 1.7B MLX 8bit** (default, GPU) / **Qwen3-ASR 0.6B** (CoreML on ANE) / **Whisper large-v3-turbo** ([speech-swift](https://github.com/soniqo/speech-swift)) | **Zero external dependencies**<br>(requires co-located `mlx.metallib`) | ~1–2.3 GB<br>`~/Library/Caches/qwen3-speech/`<br>*(supports `HF_ENDPOINT` mirror)* | Zero external deps and no daemon process; default 1.7B MLX model is the most accurate local option; `qwen3-0.6b` runs on the Neural Engine with the lowest power consumption (~375 J per 3 min) |
| **`gpu`** | **Linux / Windows / Intel Mac**<br>(Default on non-Apple-Silicon) | **ffmpeg silencedetect**<br>+ **Qwen3-ASR 1.7B GGUF Q8** | Requires `llama-server`<br>(from `llama.cpp`) | ~2.4 GB<br>`~/.cache/course2md/models/` | High-precision 1.7B Q8 quantized model; fastest throughput via Metal / CUDA / Vulkan |
| **`cpu`** | **Universal Fallback** | Same as `gpu`, with `-ngl 0` | Requires `llama-server` | ~2.4 GB<br>`~/.cache/course2md/models/` | Pure CPU execution; maximum hardware compatibility |
| **`api`** | **Cloud STT (Any platform)** | **ffmpeg silencedetect**<br>+ OpenAI-compatible `/audio/transcriptions` (e.g. OpenRouter) | **Zero local model dependencies**<br>(requires network & API key) | **None** (Cloud-hosted) | Zero disk consumption, offloads computation to cloud. *Privacy note: audio chunks are uploaded.* |
| **`npu`** | **Linux / Windows**<br>(Intel Core Ultra / AI Boost) | **ffmpeg silencedetect**<br>+ **OpenVINO Whisper Large-v3 Turbo** (default) / Base / Tiny | Requires `uv` or `python` with `openvino-genai` & NPU driver | Downloaded on demand via HuggingFace | **>6x faster than CPU**, low power, low memory (550MB vs 3.5GB CPU), high accuracy on Intel NPU |

> **CoreML Model Selection**: When using `--provider coreml`, switch models via `--asr-model qwen3-1.7b` (default, MLX on GPU), `--asr-model qwen3-0.6b` (CoreML on ANE, power-sipping), or `--asr-model whisper` (large-v3-turbo). On first use in an interactive terminal, `course2md` asks and remembers your choice in `defaults.asr_model` of `~/.config/course2md/config.toml` (a legacy `~/.config/course2md/asr_model` marker file is migrated automatically and removed).
>
> **Automatic Fallback**: On macOS, if the `coreml` backend fails during initialization or runtime, `course2md` automatically logs a warning and falls back to the `gpu` / `llama-server` pipeline to ensure task completion.

### GPU Offload Controls (`gpu` / `cpu` backends)

`--provider gpu` requests up to 99 offloaded layers and enables GPU offload for the audio encoder (mmproj) by default. Before starting, course2md checks `llama-server --list-devices`; missing devices produce setup guidance instead of silently using the CPU. On Arch/CachyOS, install a GPU backend such as `ggml-vulkan` and the matching graphics driver alongside `llama-cpp`. Use `course2md doctor` to check detection.

```bash
# Limit GPU layers and keep the audio encoder on CPU
course2md lecture.mp4 --provider gpu --gpu-layers 8 --no-mmproj-offload --transcript-source asr
# Disable GPU offload for the main model, audio encoder, and operators
course2md lecture.mp4 --provider cpu --transcript-source asr
```

Persistent settings are shared by the CLI and desktop app; explicit CLI arguments take precedence:

```toml
[defaults]
gpu_layers = 8          # 0–99; default 99
mmproj_offload = false  # Default true; --mmproj-offload enables it for one run
```

`--gpu-layers 0` alone is not CPU-only mode. With a supported llama.cpp, `--provider cpu` adds `-ngl 0 --device none --no-op-offload --no-mmproj-offload`. If an older build lacks these controls, it proceeds only when no GPU devices are detected; otherwise it asks for an updated or CPU-only build. Explicitly requesting `--no-mmproj-offload` also fails if the installed version cannot honor it.

[Issue #12](https://github.com/mizorewww/course2md/issues/12) reports GPU hangs/resets on Fedora with Radeon 780M/ROCm. Reducing layers or disabling mmproj offload may help, but neither guarantees a driver fix. Use CPU or API when affected. No unverified AMD-specific safe layer count is imposed.

Successful runs write `run.json`; failures after the output directory is established also write diagnostics with the requested backend settings, error, and actual llama-server arguments when a server was started. Earlier preflight or metadata failures do not create this file. Include it and `course2md doctor` output when reporting problems. Subtitle-based conversion skips ASR and GPU checks.


## Model Selection & Accuracy Guide

To ensure high-quality illustrated notes from lectures and technical talks, `course2md` was benchmarked thoroughly across models. **We strongly recommend Qwen3-ASR 1.7B across all platforms.**

### 1. Real-World Transcription Error & Omission Analysis (Same 3-min CS Lecture)

| Evaluation Metric | Qwen3-ASR 1.7B (Strongly Recommended) | Whisper Large-v3 Turbo | Whisper Tiny / Base |
| :--- | :--- | :--- | :--- |
| **Technical Jargon & Code-Switching** | **Flawless**: Accurately transcribes `NeoVim`, `Altair 8800`, `Computer Science`, `ICQ`, `OICQ`, `QQ`, `native speaker`, `ChatGPT`, `Web Coding`, `Codex` | **Partial mishearings**: Captures `NeoWim`, but misrecognizes `Altair 8800` as `"PCG RTIR 8800"` and `Web Coding` as `"vipcoding"` | **Severe phonetic hallucinations**: `NeoVim` misheard as "cow smell" / "pinching tail" in Chinese; most technical terms mangled |
| **Sentence Completeness** | **100% Complete**: Zero dropped clauses or truncated segment endings | **Occasional Truncation**: Fast speech at segment ends occasionally gets dropped (e.g. omitted an entire sentence on PC-to-Internet transition) | **Fragmented**: Choppy fragments |
| **Punctuation & Formatting** | **Standard & Clean**: Outputs natural commas, periods, and quotation marks (e.g. quotes around phrases and proper nouns) | **Sparse punctuation**: Mostly misses periods and quotes; runs sentences together | **Barely any valid punctuation** |

### 2. Model Trade-offs & Recommendations

| Model | Recommendation | Recommended Backend | Memory Footprint | Key Strengths | Limitations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Qwen3-ASR 1.7B** | **★★★★★<br>(Default & Recommended)** | • macOS: `--provider coreml` (default `qwen3-1.7b`, MLX on GPU) or `--provider gpu` (Metal accelerated in 13s)<br>• Linux: `--provider gpu` (CUDA) or `--provider npu`<br>• Universal: `--provider cpu` or `--provider api` | ~1.7–2.7 GB | Gold standard for Chinese & mixed-language technical lectures; flawless technical vocabulary; full punctuation; no truncated clauses | Larger download than 0.6B; MLX path uses GPU instead of the low-power ANE |
| **Qwen3-ASR 0.6B** | **★★★★☆<br>(Lightweight)** | • macOS: `--provider coreml --asr-model qwen3-0.6b` (Native Apple Neural Engine)<br>• NPU: `--provider npu --asr-model 0.6b` | ~600 MB–1.4 GB | Compact, lowest power draw on laptops on battery; zero external dependencies | Slightly lower comprehension on rare technical jargon compared to 1.7B |
| **Whisper Large-v3 Turbo** | **★★★☆☆<br>(Multilingual)** | • NPU: `--provider npu --asr-model whisper`<br>• macOS: `--provider coreml --asr-model whisper` | ~800 MB–1.5 GB | Strong for pure English or non-Chinese multilingual lectures; 12x real-time on Intel NPU | Sparse Chinese punctuation; occasional dropped clauses at segment boundaries; higher phonetic confusion on tech terms |
| **Whisper Tiny / Base** | **★☆☆☆☆<br>(Fast Pipeline Test Only)** | • NPU: `--provider npu --asr-model tiny` | <200 MB | Ultra-fast (~39x real-time, 3 min in 4s), minimal RAM | High error rate and phonetic hallucinations; not recommended for production notes |
