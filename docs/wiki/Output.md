# Output and language

[Wiki](Home.md) · [GUI](GUI.md) · [CLI](CLI.md) · [中文](Output.zh.md)

## Language

CLI help, setup prompts, common errors, and completion summaries display Chinese and English together. Commands, option names, and JSON field names stay unchanged. Run `course2md` without arguments for help; use `<command> --help` for a subcommand.

The default output shows processing stages and saved notes. `-v` adds diagnostic logs and resource details; `-vv` adds debug logs. `--quiet` suppresses progress and summaries, even when `RUST_LOG` is set. `--json` disables prompts and emits NDJSON on stdout, including parsing/configuration failures; errors exit nonzero. `--help` and `--version` remain text output.

The wizard is skipped for `--transcript-source subtitle`, `--quiet`, `--json`, and non-interactive input/output. Esc cancels setup without saving. API keys are hidden during entry. In scripts, `llm setup` requires missing settings through `--base-url`, `--api-key`, and `--model`; existing saved values can be reused. Connection-test failures keep the saved configuration and exit nonzero.

---


## Output Structure

Generated assets are organized into `out/<platform>/<title>/<id>/`:

```text
out/<platform>/<title>/<id>/
├── course.md          # Illustrated Markdown document (default)
├── course.html        # Self-contained styled HTML document (default)
├── structured.json    # Full structured data (when formats includes json)
├── frames/            # Extracted slide keyframe images
│   ├── slide_0001.jpg
│   └── ...
├── audio.wav          # Extracted audio (16kHz mono WAV)
├── timeline.jsonl     # Timestamp-aligned event stream
├── meta.json          # Video title, author, duration metadata
├── run.json           # Run provenance: version, transcript source, provider/model, stats
└── media.mp4          # Downloaded video (local input is read in-place; cleaned up by default)
```

### Completion Summary Example

On success, open one of the listed note files. Only generated files are listed; `-v` also shows peak memory and the local model directory.

```text
✓ 笔记已生成 / Notes ready
标题 / Title: Lecture
输出目录 / Output: out/local/Lecture/lecture-ID

打开以下文件查看笔记 / Open a file to read your notes:
  ✓ out/local/Lecture/lecture-ID/course.md
  ✓ out/local/Lecture/lecture-ID/course.html
截图 / Slides: out/local/Lecture/lecture-ID/frames/ (24 images)
统计 / Stats: 24 slides / 142 speech segments / 8930 characters
耗时 / Elapsed: 47s
```

---
