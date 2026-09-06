# Cloud recognition and AI editing

[Wiki](Home.md) · [GUI](GUI.md) · [CLI](CLI.md) · [中文](AI.zh.md)

## Cloud STT (`--provider api`)

`course2md` can transcribe via any OpenAI-compatible endpoint — no local GPU or ASR model download required. `base_url` accepts any custom endpoint (self-hosted gateway, DeepInfra, Groq, ...). Two request modes:

- **`transcriptions` (default)**: POST `{base_url}/audio/transcriptions`, dedicated transcription endpoint.
- **`chat`**: POST `{base_url}/chat/completions` with `input_audio` payloads — let audio-capable multimodal LLMs transcribe directly, e.g. `gpt-4o-audio-preview`, `google/gemini-2.5-flash`, `qwen2-audio`.

- **Default Provider**: OpenRouter with `qwen/qwen3-asr-flash-2026-02-10` (~$0.000035/second of audio).
- **Other Models**: Supports `openai/whisper-large-v3-turbo`, `qwen/qwen3-asr-1.7b`, etc.
- **API Key Resolution**: Reads `--asr-api-key`, config file `[asr_api].api_key`, or the `COURSE2MD_ASR_API_KEY` environment variable (`OPENROUTER_API_KEY` still accepted).

```bash
# Transcribe using OpenRouter with an environment variable
export COURSE2MD_ASR_API_KEY=sk-or-v1-xxxx
course2md https://... --provider api

# Override model or endpoint via CLI
course2md https://... --provider api --asr-api-model openai/whisper-large-v3-turbo

# Custom endpoint: point base_url at any OpenAI-compatible service
course2md https://... --provider api \
  --asr-api-base-url https://your-gateway.example.com/v1 \
  --asr-api-model whisper-large-v3

# Audio-capable multimodal LLM (chat mode)
course2md https://... --provider api --asr-api-mode chat \
  --asr-api-model google/gemini-2.5-flash
```

> **Privacy Note**: With `--provider api`, speech audio chunks are uploaded to the specified cloud endpoint for transcription. Video frames, OCR/SSIM, and VAD segmentation remain strictly local.

---


## LLM Subtitle Polishing (Optional)

`course2md` can automatically invoke a Large Language Model (LLM) after ASR transcription to proofread and refine the generated transcript.

- **Polishing Scope**: Corrects verbal tics and filler words (e.g., "um", "uh", "you know"), stuttering/repetitions, homophone typos, and technical terminology spelling. **Preserves original meaning, does not summarize, add, or translate content**.
- **Compatible Endpoints**: Any OpenAI-compatible `/chat/completions` API (e.g., DeepSeek, GLM, OpenAI, Ollama, vLLM).
- **Fault Tolerance**: Batches requests in 20-segment chunks (`temperature=0`). If a batch fails or returns invalid JSON, it automatically falls back to raw ASR text and logs a warning without halting the conversion.

> **Privacy Note**: Enabling LLM polishing uploads the transcript text to the configured LLM endpoint; with vision polishing (`vision = true`), the corresponding slide screenshots are uploaded as well. Data retention for uploaded text and screenshots is governed by that service provider. This is an independent data path from `--provider api` audio uploads — local ASR does not imply LLM text and screenshots stay local.

### Management Commands

```bash
# Interactive setup and enablement (press Enter to keep existing values; tests connectivity upon save)
course2md llm setup

# Non-interactive configuration via flags
course2md llm setup --base-url https://api.deepseek.com/v1 --api-key sk-xxxx --model deepseek-chat

# View current LLM status (API Key masked)
course2md llm status

# Disable LLM polishing while preserving configured credentials
course2md llm disable
```

### CLI Overrides at Runtime

```bash
# Force enable / disable LLM polishing for a single run
course2md https://... --llm
course2md https://... --no-llm

# Temporarily override endpoint, key, or model
course2md https://... --llm --llm-base-url https://api.deepseek.com/v1 --llm-api-key sk-xxxx --llm-model deepseek-chat

# Suppress post-run LLM suggestion hint for a single run
course2md https://... --no-llm-hint
```

---


## Video Summary (LLM, optional)

With `[llm] summarize = true` (or `course2md summarize` after setup), course2md
generates a **TL;DR / key points / timestamped outline** and inserts it at the
top of `course.md` / `course.html`:

```bash
course2md summarize out/          # summarize existing outputs (idempotent, --force to overwrite)
course2md summarize out/ -o dir/  # also export standalone <title>.summary.md files
```

- Hallucination guards: timestamped-subtitles-only input, temperature=0, structured JSON output
- Map-reduce for long videos (chunked summaries → merge)
- Reasoning-model friendly: json_object response format (auto-fallback when the
  endpoint rejects it), 16384 max_tokens, split-half retry on failures,
  4-way concurrent polishing


## Remove Credentials

Before sharing your config or committing code:

```bash
course2md remove          # clear LLM API config
course2md remove --asr    # also clear the cloud STT API key
```
