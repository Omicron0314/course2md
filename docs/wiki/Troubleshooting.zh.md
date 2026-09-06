# 故障排查

[Wiki](Home.zh.md) · [GUI](GUI.zh.md) · [CLI](CLI.zh.md) · [English](Troubleshooting.md)

## 故障排查

先跑环境体检：

```bash
course2md doctor
```

一次性报告 ffmpeg / ffprobe / yt-dlp / llama-server / uv 可用性、平台后端
（CoreML / NPU）、配置文件（含权限告警）与本地模型缓存状态。

提 issue 时请附上：

1. `course2md doctor` 完整输出
2. 输出目录中的 `run.json`（记录 provider/模型/转写来源/统计，不含凭据）
3. 所用命令行（涉密 URL 可打码）

常见问题：

| 症状 | 处理 |
| :--- | :--- |
| 网络受限下载失败 | `export HF_ENDPOINT=https://hf-mirror.com`（GGUF 与 CoreML 下载均生效；下载失败的报错也会提示该镜像） |
| 换模型后转写混杂 | 1.0 起旧 checkpoint 自动作废；也可 `--no-resume` 强制重算 |
| `--no-download` 删了我的视频 | 1.0 已修复——非本次运行下载的文件永不删除 |
| NPU 上英文课被转成中文 | 1.0 已修复——语言改为自动检测，不再强制中文 |
| 想直接用平台字幕 | 1.0 默认行为（`--transcript-source auto`）；强制字幕用 `--transcript-source subtitle` |
