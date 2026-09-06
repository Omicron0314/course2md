# 输出与语言

[Wiki](Home.zh.md) · [GUI](GUI.zh.md) · [CLI](CLI.zh.md) · [English](Output.md)

## 语言

命令行帮助、设置向导、常见错误和完成摘要采用中英同屏提示。命令、参数名称和 JSON 字段保持不变。不带参数运行 `course2md` 查看入门帮助，使用 `<command> --help` 查看子命令。

默认输出处理阶段和笔记位置；`-v` 增加诊断日志及资源详情，`-vv` 增加调试信息。`--quiet` 隐藏进度与完成摘要，即使设置了 `RUST_LOG` 也只报告错误。`--json` 禁用交互，在 stdout 输出 NDJSON，包括参数和配置错误；失败返回非零退出码。`--help` 和 `--version` 仍输出文本。

`--transcript-source subtitle`、`--quiet`、`--json` 和非交互输入输出会跳过设置向导。Esc 取消且不保存配置，密钥输入隐藏。脚本调用 `llm setup` 时，通过 `--base-url`、`--api-key`、`--model` 补全缺失设置，也可复用已保存值。连接测试失败会保留配置，并返回非零退出码。

---


## 输出目录结构

转换产物按 `out/<平台>/<标题>/<编号>/` 格式自动归档：

```text
out/<平台>/<标题>/<编号>/
├── course.md          # 图文混排 Markdown 文档（默认生成）
├── course.html        # 独立排版 HTML 页面（默认生成）
├── structured.json    # 结构化数据（指定 --formats 包含 json 时生成）
├── frames/            # 文稿中引用的幻灯片/关键帧截图
│   ├── slide_0001.jpg
│   └── ...
├── audio.wav          # 提取的音频（16kHz 单声道 WAV）
├── timeline.jsonl     # 带时间戳对齐的原始识别序列
├── meta.json          # 视频标题、作者、时长等元数据
├── run.json           # 本次运行溯源：版本、转写来源、provider/模型、统计
└── media.mp4          # 下载的视频（本地文件输入时不重复复制；默认转换完成后自动清理）
```

### 完成摘要输出示例

任务完成后，终端会列出笔记路径、统计信息和耗时；使用 `-v` 可查看峰值内存和模型目录：

```text
✓ 笔记已生成 / Notes ready
标题 / Title: 课程
输出目录 / Output: out/local/课程/lecture-ID

打开以下文件查看笔记 / Open a file to read your notes:
  ✓ out/local/课程/lecture-ID/course.md
  ✓ out/local/课程/lecture-ID/course.html
截图 / Slides: out/local/课程/lecture-ID/frames/ (24 images)
统计 / Stats: 24 slides / 142 speech segments / 8930 characters
耗时 / Elapsed: 47s
```

---
