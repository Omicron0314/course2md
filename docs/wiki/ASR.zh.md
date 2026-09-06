# 语音识别与模型

[Wiki](Home.zh.md) · [GUI](GUI.zh.md) · [CLI](CLI.zh.md) · [English](ASR.md)

## 识别后端（ASR Backends）

`course2md` 提供多种识别后端，可通过 `--provider <后端>` 或在配置文件中指定：

| 后端 (`--provider`) | 适用平台与默认策略 | 核心架构与模型 | 外部依赖 | 首次下载与缓存路径 | 特点 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`coreml`** | **macOS Apple Silicon**<br>(预编译包默认) | **Silero VAD v6.2.1 CoreML** (ANE)<br>+ **Qwen3-ASR 1.7B MLX 8bit**（默认，走 GPU）/ **Qwen3-ASR 0.6B**（CoreML 走 ANE）/ **Whisper large-v3-turbo** ([speech-swift](https://github.com/soniqo/speech-swift)) | **零外部依赖**<br>(仅需同目录 `mlx.metallib`) | 约 1~2.3GB<br>`~/Library/Caches/qwen3-speech/`<br>*(支持 `HF_ENDPOINT` 镜像)* | 零外部依赖、无子进程；默认 1.7B MLX 模型最准；`qwen3-0.6b` 走神经网络引擎 (ANE)，功耗极低（3 分钟约 375 J） |
| **`gpu`** | **Linux / Windows / Intel Mac**<br>(非 Apple Silicon 默认) | **ffmpeg silencedetect**<br>+ **Qwen3-ASR 1.7B GGUF Q8** | 需要 `llama-server`<br>(由 `llama.cpp` 提供) | 约 2.4GB<br>`~/.cache/course2md/models/` | 1.7B 高精度量化模型，支持 Metal / CUDA / Vulkan 等显卡加速，吞吐极高 |
| **`cpu`** | **通用兜底** | 同 `gpu`，严格禁用 GPU 卸载 | 需要 `llama-server` | 约 2.4GB<br>`~/.cache/course2md/models/` | 纯 CPU 计算，兼容性最高 |
| **`api`** | **云端 STT（跨平台通用）** | **ffmpeg silencedetect**<br>+ OpenAI 兼容 `/audio/transcriptions` 端点（如 OpenRouter） | **零本地模型依赖**<br>(需网络与 API Key) | **无**（云端托管） | 零磁盘模型占用，低配置设备友好。*隐私提示：音频切片将上传云端。* |
| **`npu`** | **Linux / Windows**<br>(Intel Core Ultra / AI Boost) | **ffmpeg silencedetect**<br>+ **OpenVINO Whisper Large-v3 Turbo** (默认) / Base / Tiny | 需要 `uv` 或 `python` 带 `openvino-genai` 与 NPU 驱动 | 按需自 HuggingFace 下载 | **比纯 CPU 快 6 倍以上**，极低功耗，显存/内存节省 84%（550MB vs 3.5GB） |

> **CoreML 模型切换**：使用 `--provider coreml` 时，可通过 `--asr-model qwen3-1.7b`（默认，MLX 走 GPU）、`--asr-model qwen3-0.6b`（CoreML 走 ANE，省电）或 `--asr-model whisper`（large-v3-turbo）切换。首次在交互式终端使用且未配置时，程序会提示选择并记忆至 `~/.config/course2md/config.toml` 的 `defaults.asr_model`（旧的 `~/.config/course2md/asr_model` marker 文件会自动迁移后删除）。
>
> **自动回落机制**：在 macOS 上如果 `coreml` 后端初始化或运行失败，系统会自动给出警告并无缝回退至 `gpu` / `llama-server` 模式，确保转换任务顺利完成。

### GPU 卸载控制（`gpu` / `cpu` 后端）

`--provider gpu` 默认请求最多 99 层 GPU 卸载，并启用音频编码器（mmproj）的 GPU 卸载。启动前会检查 `llama-server --list-devices`；没有可用设备时给出安装提示，避免静默使用 CPU。Arch/CachyOS 除 `llama-cpp` 外还需 GPU 后端，例如 `ggml-vulkan`，以及对应显卡驱动。可用 `course2md doctor` 检查设备。

```bash
# 限制主模型 GPU 层数，并让音频编码器留在 CPU
course2md lecture.mp4 --provider gpu --gpu-layers 8 --no-mmproj-offload --transcript-source asr
# 禁用主模型、音频编码器和算子的 GPU 卸载
course2md lecture.mp4 --provider cpu --transcript-source asr
```

持久配置与 CLI/桌面端共享，命令行参数优先：

```toml
[defaults]
gpu_layers = 8          # 0–99，默认 99
mmproj_offload = false  # 默认 true；--mmproj-offload 可临时重新启用
```

`--gpu-layers 0` 本身不等于纯 CPU。`--provider cpu` 会在支持的 llama.cpp 上附加 `-ngl 0 --device none --no-op-offload --no-mmproj-offload`。旧版缺少这些控制时，只有确认没有 GPU 设备才继续；否则提示更新 llama.cpp 或使用 CPU-only 构建。明确要求 `--no-mmproj-offload` 而当前版本不支持时也会报错。

[#12](https://github.com/mizorewww/course2md/issues/12) 报告了 Fedora + Radeon 780M/ROCm 下的 GPU hang/reset。减少层数或关闭 mmproj 卸载是可尝试的调节方式，不能保证解决驱动问题；遇到此类问题可改用 CPU 或 API。项目未设置未经验证的 AMD 专用“安全层数”。

成功转换会写入 `run.json`；输出目录确定后的处理失败也会写入诊断记录，其中包含请求的后端配置、错误及已启动 llama-server 的实际参数。更早的预检或元数据失败不会生成该文件。报告问题时可附上此文件和 `course2md doctor` 输出。使用字幕时会跳过 ASR 和 GPU 检查。


## 模型选型与错漏分析指南（Model Selection & Accuracy Guide）

为了保证网课与学术视频转换的高质量，`course2md` 在多款模型间进行了严格的实测对照。**所有平台均首推采用 Qwen3-ASR 1.7B**。

### 1. 真实评测错漏分析（以同一段 3 分钟大学计算机课程为例）

| 维度 | Qwen3-ASR 1.7B（全平台首选推荐） | Whisper Large-v3 Turbo | Whisper Tiny / Base |
| :--- | :--- | :--- | :--- |
| **中英混合专业词汇** | **极准**：精准识别 `NeoVim`、`Altair 8800`、`Computer Science`、`ICQ`、`OICQ`、`QQ`、`native speaker`、`ChatGPT`、`Web Coding`、`Codex` | **存在误判**：识别出 `NeoWim`，但将 `Altair 8800` 误为 `"PCG RTIR 8800"`，`Web Coding` 误为 `"vipcoding"` | **严重幻觉**：`NeoVim` 严重错认为“牛味”、“捏尾巴”；专业术语大部分无法辨识 |
| **句子完整度** | **100% 完整**：无漏句、无截断，说话人语速较快时依然完整留存 | **偶发截断**：长分段末尾偶发丢失整句（例如漏掉“啊，整理一次，从PC到互联网...”） | **分段碎裂**：多处短句残缺 |
| **标点符号规范** | **规范完整**：全自动输出符合中文语法的逗号、句号、双引号（如“AI替我上大学”、“hello”） | **标点缺失**：句号大面积缺失，长难句连成一片 | **基本无有效标点** |

### 2. 几个主要模型的优劣与适用场景

| 模型 | 推荐级别 | 推荐运行方式 | 显存/内存占用 | 核心优势 | 劣势与注意事项 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Qwen3-ASR 1.7B** | **★★★★★<br>(强烈推荐)** | • macOS: `--provider coreml`（默认 `qwen3-1.7b`，MLX 走 GPU）或 `--provider gpu` (Metal 加速，仅需 13 秒)<br>• Linux: `--provider gpu` (CUDA) 或 `--provider npu`<br>• 通用: `--provider cpu` 或 `--provider api` | ~1.7GB~2.7GB | 中文及技术课程整体表现更好；标点较完整，专有名词更稳 | 模型体积略大于 0.6B；MLX 路径走 GPU，放弃 ANE 低功耗 |
| **Qwen3-ASR 0.6B** | **★★★★☆<br>(极致高能效)** | • macOS: `--provider coreml --asr-model qwen3-0.6b` (Apple Neural Engine 原生)<br>• NPU: `--provider npu --asr-model 0.6b` | ~600MB~1.4GB | 体积小、在轻薄本和电池模式下能效极高；纯本地零外部依赖 | 生僻复杂技术词理解略逊于 1.7B 满血版 |
| **Whisper Large-v3 Turbo** | **★★★☆☆<br>(纯英文/小语种)** | • NPU: `--provider npu --asr-model whisper`<br>• macOS: `--provider coreml --asr-model whisper` | ~800MB~1.5GB | 纯英文或非中文多语种识别能力优秀；OpenVINO NPU 上达 12x 实时加速 | 中文标点欠缺；语速快时偶发句尾吞词；技术词音近误判率高 |
| **Whisper Tiny / Base** | **★☆☆☆☆<br>(仅供测试)** | • NPU: `--provider npu --asr-model tiny` | <200MB | 极速（39x 实时，3分钟仅需4秒），极低显存 | 严重音近幻觉，不建议用于正式讲义 |
