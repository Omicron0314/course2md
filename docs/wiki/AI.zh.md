# 云端识别与 AI 整理

[Wiki](Home.zh.md) · [GUI](GUI.zh.md) · [CLI](CLI.zh.md) · [English](AI.md)

## 云端 STT 支持 (`--provider api`)

`course2md` 支持接入任意 OpenAI 兼容端点，无需本地显卡与大模型下载。`base_url` 可指向任何自定义端点（自建网关、DeepInfra、Groq 等）。两种请求模式：

- **`transcriptions`（默认）**：POST `{base_url}/audio/transcriptions`，专用转录端点。
- **`chat`**：POST `{base_url}/chat/completions`（`input_audio` 音频输入），让支持音频的多模态 LLM 直接转录，如 `gpt-4o-audio-preview`、`google/gemini-2.5-flash`、`qwen2-audio`。

- **推荐服务**：OpenRouter 托管的 `qwen/qwen3-asr-flash-2026-02-10`（约 $0.000035 / 秒音频）。
- **兼容模型**：支持 `openai/whisper-large-v3-turbo`、`qwen/qwen3-asr-1.7b` 等。
- **密钥获取与配置**：可通过 `--asr-api-key`、配置文件 `[asr_api].api_key` 或 `COURSE2MD_ASR_API_KEY` 环境变量传入（兼容旧名 `OPENROUTER_API_KEY`）。

```bash
# 通过环境变量使用 OpenRouter 转写
export COURSE2MD_ASR_API_KEY=sk-or-v1-xxxx
course2md https://... --provider api

# 命令行即时覆盖模型
course2md https://... --provider api --asr-api-model openai/whisper-large-v3-turbo

# 自定义端点：base_url 指向任何 OpenAI 兼容服务
course2md https://... --provider api \
  --asr-api-base-url https://your-gateway.example.com/v1 \
  --asr-api-model whisper-large-v3

# 音频多模态 LLM（chat 模式）
course2md https://... --provider api --asr-api-mode chat \
  --asr-api-model google/gemini-2.5-flash
```

> **隐私提示**：使用 `--provider api` 时，语音切片将上传至所配置的云端服务完成转写；视频截图、SSIM 画面分析与 VAD 静音切分仍全部在本地执行。

---


## LLM 字幕润色（可选）

`course2md` 支持在 ASR 转写完成后，调用大语言模型（LLM）对字幕文本进行自动化校对与润色。

- **润色目标**：修正语气词/口头禅（如「呃」、「嗯」、「这个那个」等）、重复字词、明显的同音错别字与专有名词拼写；**不增删实质内容、不翻译、不改变原意**。
  - *示例*：`我我干了什么呢？我在，我这是我的Neo Vim` → `我干了什么呢？我在，这是我的Neo Vim`
- **兼容接口**：支持任意 OpenAI 兼容的 `/chat/completions` 端点（如 DeepSeek、GLM、OpenAI、Ollama、vLLM 等）。
- **容错保证**：按 20 段语音合并批次并发起请求（`temperature=0`）。若某批次请求失败或响应解析异常，将自动回退保留 ASR 原始文本并给出警告，**绝不阻断整体转换流程**。

> **隐私提示**：开启 LLM 润色会将转写文本上传至所配置的 LLM 服务；开启视觉润色（`vision = true`）还会随请求上传对应的幻灯片截图。文本与截图的数据保留政策由该服务商决定。这与 `--provider api` 的语音上传是相互独立的数据路径——「ASR 在本地运行」不代表 LLM 的文本和截图也留在本地。

### 快捷管理命令

```bash
# 交互式配置并开启（提示输入，按回车保留已配置项，保存后自动测试连通性）
course2md llm setup

# 也可以直接通过命令行参数配置
course2md llm setup --base-url https://api.deepseek.com/v1 --api-key sk-xxxx --model deepseek-chat

# 查看当前 LLM 配置状态（API Key 自动脱敏打码）
course2md llm status

# 暂时关闭 LLM 润色功能（保留已配置的凭据与端点）
course2md llm disable
```

### 运行时命令行覆盖

```bash
# 单次运行强制开启 / 关闭 LLM 润色
course2md https://... --llm
course2md https://... --no-llm

# 临时指定其他模型或端点
course2md https://... --llm --llm-base-url https://api.deepseek.com/v1 --llm-api-key sk-xxxx --llm-model deepseek-chat

# 单次运行关闭结束时的 LLM 开启提示
course2md https://... --no-llm-hint
```

---


## 视频总结（LLM，可选）

开启 `[llm] summarize = true`（或配置后用 `course2md summarize`）后，
course2md 会为文稿生成 **TL;DR / 核心要点 / 带时间戳大纲**，插入
`course.md` / `course.html` 开头：

```bash
course2md summarize out/          # 为已有输出生成总结（幂等，--force 覆盖）
course2md summarize out/ -o dir/  # 另导出独立 <标题>.summary.md
```

- 幻觉防护：仅以带时间戳字幕为输入、temperature=0、JSON 结构化输出
- 超长视频自动 map-reduce（分段总结 → 合并）
- 推理模型（DeepSeek V4 Flash 等）兼容：json_object 响应格式（端点不支持时自动降级）、
  16384 max_tokens、失败批次拆半递归重试、润色 4 路并发


## 清除凭据

分享配置或提交代码前：

```bash
course2md remove          # 清除 LLM API 配置
course2md remove --asr    # 同时清除云端 STT 的 API Key
```
