# Benchmarks

[Wiki](Home.md) · [GUI](GUI.md) · [CLI](CLI.md) · [中文](Benchmarks.zh.md)

## Benchmarks & Power Metrics

Measured on Apple Silicon (arm64) running a **3-minute** 1080p recorded lecture clip with `powermetrics` hardware sampling (idle baseline ≈ 1.9 W):

| Backend (`--provider`) | Wall Time | Avg Power (CPU / GPU / ANE) | Peak Memory | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **`coreml` + qwen3-0.6b** | 47 s | 6.7 W / 0.2 W / **3.5 W** | 1.41 GB in-proc | **Lowest power** — Neural Engine does the heavy lifting; best on battery; zero external dependencies |
| **`coreml` + whisper-turbo** | 87 s | 15.3 W / 0.3 W / 0.4 W | 1.51 GB in-proc | Whisper large-v3-turbo on CoreML; decoder mostly on CPU for short segments |
| **`gpu` (llama.cpp Metal)** | **13 s** | 4.7 W / **16.0 W** / — | 26 MB + 3.3 GB child | **Fastest**; GPU bursts; needs `llama-server` (Qwen3-ASR 1.7B Q8) |
| **`cpu` (llama.cpp)** | 26 s | **21.2 W** / 0.6 W / — | 26 MB + 4.8 GB child | Universal fallback; high CPU power |
| **`api` (cloud STT)** | ~10 s | < 1 W | negligible | Audio uploaded to provider; speed depends on network |
| **`npu` (Intel Core Ultra)** | **16 s** | NPU hardware acceleration | 18 MB + 557 MB child | **>6x faster than CPU** on Intel Core Ultra laptops; Whisper Large-v3 Turbo |

> **Note on the `coreml` default model**: the measurements above were taken with the former 0.6B CoreML default. The current default **`qwen3-1.7b`** (Qwen3-ASR 1.7B MLX 8bit, running on GPU) is both more accurate and faster per upstream benchmarks (WER 1.52% vs 3.02%, RTF 0.033 vs 0.098), at the cost of roughly 2× peak memory (RSS ~2.7 GB vs ~1.4 GB) and giving up the ANE low-power path. Pick `--asr-model qwen3-0.6b` when battery life matters most.

👉 See the comprehensive [macOS Benchmark Report](../BENCHMARKS.md) for full methodology, energy breakdowns, and reproduction scripts.

---
