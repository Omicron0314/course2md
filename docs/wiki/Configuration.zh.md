# 配置参考

[Wiki](Home.zh.md) · [GUI](GUI.zh.md) · [CLI](CLI.zh.md) · [English](Configuration.md)

## 配置文件（Configuration）

为了避免每次输入冗长的命令行参数，`course2md` 提供了完善的全局配置文件支持。

### 文件路径
- **macOS / Linux**：`~/.config/course2md/config.toml`（遵循 `$XDG_CONFIG_HOME` 规范）
- **Windows**：`%APPDATA%\course2md\config.toml`

### 优先级规则
**命令行参数 (CLI Flags) > 配置文件 (config.toml) > 内置默认值 (Built-in Defaults)**

### 便捷配置命令

```bash
# 1. 初始化生成带完整注释的配置模板（文件已存在时加 --force 可覆盖）
course2md config init

# 2. 查看当前配置文件路径及生效的默认设置
course2md config show
```

### 配置项参考

```toml
# ~/.config/course2md/config.toml

[defaults]
# 输出根目录（其下按 平台/标题/编号 自动归类）
out = "out"

# 画面变化 SSIM 相似度阈值（0.0 ~ 1.0），数值越高越敏感、截图越多
similarity = 0.85

# 画面采样检查间隔（秒）
sample_interval = 1.0

# 新截图触发后的冷却防抖间隔（秒）
cooldown = 10.0

# 感兴趣区域（ROI），格式如 "40%,0%-100%,100%"，留空则比较全屏
# roi = "40%,0%-100%,100%"

# ASR 识别线程数（供本地 llama.cpp 使用）
threads = 4

# 识别后端：coreml（macOS Apple Silicon 推荐）| gpu | cpu | api
# provider = "coreml"

# CoreML 识别模型选择：qwen3-1.7b（默认，MLX 走 GPU）| qwen3-0.6b（CoreML 走 ANE，省电）| whisper（large-v3-turbo）
# asr_model = "qwen3-1.7b"

# 单段语音最长切分秒数
max_speech = 20.0

# 默认生成的文稿格式，支持 md, html, json
formats = ["md", "html"]

# llama.cpp GGUF 模型目录（留空使用默认缓存）
# model_dir = "~/.cache/course2md/models"

# 是否保留下载的原始 media.mp4 文件
keep_video = false

[asr_api]
# 云端 STT 配置（--provider api 时使用）
# base_url 可指向任何 OpenAI 兼容端点（自建网关、DeepInfra、Groq 等均可）
#mode = "transcriptions"   # transcriptions = POST {base_url}/audio/transcriptions（默认，专用转录端点）
                           # chat = POST {base_url}/chat/completions（支持音频输入的多模态 LLM，
                           #        如 gpt-4o-audio-preview、google/gemini-2.5-flash、qwen2-audio）
base_url = "https://openrouter.ai/api/v1"
api_key = "sk-or-v1-xxxxxxxx"
model = "qwen/qwen3-asr-flash-2026-02-10"
# OpenRouter 上其他常用模型：openai/whisper-large-v3-turbo, qwen/qwen3-asr-1.7b

[llm]
# 是否默认开启 LLM 字幕润色（默认 false，运行 course2md llm setup 可交互式开启）
enabled = false

# OpenAI 兼容 API 地址（如未包含协议头会自动补全 https://）
base_url = "https://api.deepseek.com/v1"

# API 密钥（文件权限在 Unix 上自动设置为 0600）
api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# 使用的模型名称
model = "deepseek-chat"

# 自定义校对提示词（留空则使用内置的高质量校对 Prompt）
prompt = ""

# 是否永久关闭任务结束时的 LLM 开启提示（默认 false）
disable_hint = false

# 视觉润色：润色请求附对应幻灯片截图，辅助纠正术语拼写（需模型支持图片输入，默认 false）
#vision = false

# 润色并发数（Section 间相互独立；自建网关/代理可调高）
#concurrency = 8
```

---
