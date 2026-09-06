# 性能评测

[Wiki](Home.zh.md) · [GUI](GUI.zh.md) · [CLI](CLI.zh.md) · [English](Benchmarks.md)

## 性能与功耗实测（Benchmarks）

在 Apple Silicon (arm64) 上运行 **3 分钟** 1080p 教学课件视频实测，通过 `powermetrics` 采集硬件功率（整机空闲基线 ≈ 1.9 W）：

| 识别后端 (`--provider`) | 总耗时 | 平均功率 (CPU / GPU / ANE) | 峰值内存 | 说明 |
| :--- | :--- | :--- | :--- | :--- |
| **`coreml` + qwen3-0.6b** | 47 s | 6.7 W / 0.2 W / **3.5 W** | 1.41 GB 进程内 | **功耗最低**：神经网络引擎扛主力，电池场景首选；零外部依赖 |
| **`coreml` + whisper-turbo** | 87 s | 15.3 W / 0.3 W / 0.4 W | 1.51 GB 进程内 | Whisper large-v3-turbo CoreML；短分段下解码器主要在 CPU |
| **`gpu`**（llama.cpp Metal） | **13 s** | 4.7 W / **16.0 W** / — | 26 MB + 3.3 GB 子进程 | **最快**：GPU 峰值高；需 `llama-server`（Qwen3-ASR 1.7B Q8） |
| **`cpu`**（llama.cpp） | 26 s | **21.2 W** / 0.6 W / — | 26 MB + 4.8 GB 子进程 | 通用兜底；CPU 功耗高 |
| **`api`**（云端 STT） | ~10 s | < 1 W | 可忽略 | 音频会上传；速度取决于网络 |
| **`npu`**（Intel Core Ultra） | **16 s** | NPU 硬件加速 | 18 MB + 557 MB 子进程 | **比纯 CPU 快 6 倍**（3 分钟音频 15 秒识别），低功耗，Whisper Large-v3 Turbo |

> **关于 `coreml` 默认模型的说明**：上表数据是在旧默认 0.6B CoreML 模型下实测的。当前默认 **`qwen3-1.7b`**（Qwen3-ASR 1.7B MLX 8bit，走 GPU）按上游 benchmark 更准也更快（WER 1.52% vs 3.02%，RTF 0.033 vs 0.098），代价是峰值内存约 2 倍（RSS 约 2.7GB vs 1.4GB），且不再走 ANE 低功耗路径。电池优先场景可显式选择 `--asr-model qwen3-0.6b`。

👉 详见完整的 [macOS 性能与功耗基准报告](../BENCHMARKS.md)（含测试方法论、详细能耗拆解与复现脚本）。

---
