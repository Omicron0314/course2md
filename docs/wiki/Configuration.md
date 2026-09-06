# Configuration reference

[Wiki](Home.md) · [GUI](GUI.md) · [CLI](CLI.md) · [中文](Configuration.zh.md)

## Configuration

To avoid passing repetitive command-line arguments, `course2md` provides a global TOML configuration file.


## Configuration Path
- **macOS / Linux**: `~/.config/course2md/config.toml` (follows `$XDG_CONFIG_HOME`)
- **Windows**: `%APPDATA%\course2md\config.toml`

### Priority Hierarchy
**CLI Flags > Configuration File (config.toml) > Built-in Defaults**


## Configuration Management Commands

```bash
# 1. Generate an annotated configuration template (use --force to overwrite existing)
course2md config init

# 2. Display the configuration path and effective default settings
course2md config show
```


## Configuration File Structure

```toml
# ~/.config/course2md/config.toml

[defaults]
# Output root directory (structured as <out>/<platform>/<title>/<id>/)
out = "out"

# Frame similarity SSIM threshold (0.0 to 1.0; lower value = more slides captured)
similarity = 0.85

# Frame sampling check interval in seconds
sample_interval = 1.0

# Cooldown time (seconds) after a new slide is captured before capturing again
cooldown = 10.0

# Region of Interest (ROI), e.g. "40%,0%-100%,100%"; empty compares full frame
# roi = "40%,0%-100%,100%"

# ASR transcription thread count (for local llama.cpp)
threads = 4

# Inference backend: coreml (macOS Apple Silicon) | gpu | cpu | api
# provider = "coreml"

# CoreML model variant: qwen3-1.7b (default, MLX on GPU) | qwen3-0.6b (CoreML on ANE, low power) | whisper (large-v3-turbo)
# asr_model = "qwen3-1.7b"

# Maximum speech segment duration in seconds before splitting
max_speech = 20.0

# Output document formats: md, html, json
formats = ["md", "html"]

# llama.cpp GGUF model directory (leave commented for default cache)
# model_dir = "~/.cache/course2md/models"

# Keep downloaded media.mp4 video file after processing
keep_video = false

[asr_api]
# Cloud STT settings (used when --provider api)
# base_url can point to any OpenAI-compatible endpoint (self-hosted gateway, DeepInfra, Groq, ...)
#mode = "transcriptions"   # transcriptions = POST {base_url}/audio/transcriptions (default, dedicated STT endpoint)
                           # chat = POST {base_url}/chat/completions (audio-capable multimodal LLMs,
                           #        e.g. gpt-4o-audio-preview, google/gemini-2.5-flash, qwen2-audio)
base_url = "https://openrouter.ai/api/v1"
api_key = "sk-or-v1-xxxxxxxx"
model = "qwen/qwen3-asr-flash-2026-02-10"
# Other popular models on OpenRouter: openai/whisper-large-v3-turbo, qwen/qwen3-asr-1.7b

[llm]
# Enable LLM subtitle polishing by default (default: false; run `course2md llm setup` to configure)
enabled = false

# OpenAI-compatible API endpoint (auto-prefixes https:// if omitted)
base_url = "https://api.deepseek.com/v1"

# API Key (file permissions automatically restricted to 0600 on Unix)
api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# Model identifier
model = "deepseek-chat"

# Custom prompt (leave empty to use high-quality built-in proofreading prompt)
prompt = ""

# Permanently suppress the post-run LLM suggestion hint (default: false)
disable_hint = false

# Vision polishing: attach the slide screenshot to each request to correct
# terminology spelling (requires a vision-capable model; default: false)
#vision = false

# Polishing concurrency (sections are independent; raise for self-hosted gateways)
#concurrency = 8
```

---
