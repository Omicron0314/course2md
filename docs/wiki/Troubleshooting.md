# Troubleshooting

[Wiki](Home.md) · [GUI](GUI.md) · [CLI](CLI.md) · [中文](Troubleshooting.zh.md)

## Troubleshooting

Run the environment check first:

```bash
course2md doctor
```

It reports ffmpeg / ffprobe / yt-dlp / llama-server / uv availability, platform
backends (CoreML / NPU), config-file validity (including permission warnings),
and the local model cache state.

When opening an issue, please attach:

1. The full output of `course2md doctor`
2. The `run.json` file from the output directory (records provider, model,
   transcript source, and stats — no credentials)
3. The command line you used (redact URLs if needed)

Common fixes:

| Symptom | Fix |
| :--- | :--- |
| Download fails on restricted networks | `export HF_ENDPOINT=https://hf-mirror.com` (honored by both GGUF and CoreML downloads; download errors print this hint too) |
| Transcripts look mixed/inconsistent after switching models | Pre-1.0 checkpoints are discarded automatically; rerun with `--no-resume` to force a clean pass |
| `--no-download` deleted my video | Fixed in 1.0 — files not downloaded by the current run are never removed |
| English course transcribed as Chinese on NPU | Fixed in 1.0 — language is auto-detected; force nothing |
| Prefer platform subtitles over local ASR | Default behavior in 1.0 (`--transcript-source auto`); force with `--transcript-source subtitle` |
