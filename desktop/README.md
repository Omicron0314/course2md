# course2md 原生桌面端

基于 GPUI 与 GPUI Component 的 macOS、Windows、Linux 客户端。转换使用同包 CLI，支持链接和本地视频、字幕优先、各 ASR 后端、进度/取消、课程搜索、图文笔记阅读，以及共享配置文件。

## 安装与首次使用

普通用户请阅读 [GUI 安装指南](https://github.com/mizorewww/course2md/wiki/%E5%AE%89%E8%A3%85%E6%A1%8C%E9%9D%A2%E5%BA%94%E7%94%A8)（[English](https://github.com/mizorewww/course2md/wiki/GUI-Installation)）。Homebrew GUI 使用 `brew install --cask mizorewww/tap/course2md-gui`；AUR GUI 使用 `yay -S course2md-gui-bin`。首次启动按向导完成设置。

本页其余内容面向开发者，无需为使用应用安装 Rust 或执行构建命令。所有用户文档见 [Wiki](https://github.com/mizorewww/course2md/wiki)。

## 开发

需要 Rust stable、Python 3.11+、Git。macOS 需 Xcode Command Line Tools；Windows 需 Visual Studio C++ Build Tools 和 LLVM；Linux 系统依赖见 `.github/workflows/desktop.yml`。处理视频需要 ffmpeg/ffprobe，远程链接还需要 yt-dlp。GPU/CPU 识别需 llama-server；Apple 原生、Intel NPU 和 API 的要求与 CLI 相同。

```sh
python3 desktop/scripts/sources.py
cargo build
cargo build --manifest-path desktop/Cargo.toml
# 开发时指定刚构建的引擎；Windows PowerShell 使用 $env:COURSE2MD_BIN
COURSE2MD_BIN="$PWD/target/debug/course2md" cargo run --manifest-path desktop/Cargo.toml
```

`sources.py` 默认先 fast-forward pull `~/Developer/zed` 和 `~/Developer/gpui-component` 的 main，再建立 `desktop/.deps` 下的独立工作树。可通过 `--developer-dir` 指定仓库根目录；`--no-pull` 仅复用本次已更新的源码。开发不使用版本号或固定 commit。两个原始工作区必须干净，脚本不会替你丢弃修改。

组件主线现属于 GPUI Kit，使用重新发布的 `gpui-pre` 包名。准备脚本只在独立工作树中将依赖映射到 Zed GPUI，并让组件宏兼容原始 `gpui` 包名；不改动开发者原始工作区。上游布局变更时脚本会明确失败，要求检查兼容调整。

应用支持 `COURSE2MD_BIN`、同目录 CLI、PATH 三种引擎位置。macOS 从 Finder 启动也会补充 Homebrew 工具路径。配置位置与 CLI 相同，首次桌面使用默认保存到 `~/Documents/course2md`。

## 本机打包

```sh
python3 -m pip install -r desktop/scripts/requirements-packaging.txt
python3 desktop/scripts/package.py --debug
```

产物位于 `desktop/target/packages/`。macOS 为包含 CLI 和 MLX Metal 库的 `.app`；Windows 为两份 `.exe`；Linux 为两份可执行文件以及桌面入口。Windows/Linux 解压后保留两份程序在同一目录。本机默认 ad-hoc 签名；发布 CI 使用已有 Developer ID 与 Apple API 凭据签名、公证并生成 DMG。macOS ZIP 使用 ditto 保留签名所需的符号链接。

macOS DMG 使用 dmgbuild 固定 Finder 窗口、Retina 背景和拖动位置，窗口只显示应用与 Applications 快捷方式；许可证与源码版本信息保存在应用的 Resources 内。打包后自动挂载检查布局、图标、快捷方式与应用签名，不依赖 Finder GUI。设计与实机预览见 [安装界面](assets/dmg/README.md)。

Windows 构建将多尺寸 ICO 嵌入 GUI 程序的资源编号 `1`，与 GPUI 读取的编号一致。打包会检查成品 `.exe` 的每个图标尺寸及内容；缺失或过期会直接失败。应用图标的 Icon Composer 源文件与导出方式见 [图标设计](assets/icon-design/README.md)。

## 发布

完成开发、测试和实际界面验收后冻结**当时使用的**源码：

```sh
python3 desktop/scripts/sources.py --freeze
# 一起提交 sources.lock.json 和 desktop/Cargo.lock
python3 desktop/scripts/sources.py --locked
python3 desktop/scripts/package.py
```

发布命令核对实际工作树与冻结记录，并使用 Cargo `--locked`。普通开发命令仍继续追踪 main。CI 为三个系统分别构建；发布使用冻结记录，PR 构建使用主线。可以手动运行 release 工作流并指定版本，全部构建成功后再创建对应 tag 和 GitHub Release。

预发布使用 `2.0.0-alpha.1` 格式，并同步更新两个 Cargo.toml、锁文件和 `docs/releases/v版本.md`。release 工作流会设置 GitHub Pre-release，保留稳定版 Latest。Homebrew 自动更新独立的 `course2md-alpha` / `course2md-gui@alpha` 通道；AUR 跳过预发布。macOS 包将完整版本保存在 `Course2mdVersion`，使用符合 Apple 格式的数字短版本及开发后缀（如 `2.0.0` / `2.0.0a1`），应用「关于」和 CLI 显示完整 SemVer。

## 验证

```sh
cargo test --features integration
cargo test --manifest-path desktop/Cargo.toml
```

实际操作记录与已知边界见 [工作区验收](../docs/DESKTOP-WORKSPACE-ACCEPTANCE.md)。布局、状态与动效约定见 [设计规范](../docs/DESKTOP-DESIGN-SYSTEM.md)。开发验收使用独立 `XDG_CONFIG_HOME`，不修改个人 API 配置。

首次使用流程、能力检测与对比度验收见 [首次设置引导](../docs/DESKTOP-FIRST-RUN.md)。


## 界面与操作

- 首次打开自动显示设置引导，自动选择本机识别方式并选择笔记目录；完成或稍后设置后不再自动出现，可从「设置 → 通用」重新打开。缺少视频处理工具时直接进入安装帮助。
- 添加页粘贴在线链接或选择本地视频，确认封面后选择文件夹并生成；字幕策略、识别方式、导出格式与 AI 整理集中在可展开的转换选项中。
- 生成、取消等操作固定在顶部工具栏；底栏仅在任务运行时显示真实进度，并可进入任务详情。
- 任务分别显示真实工作项、并行数量及下载/转写进度和 ETA；未知进度显示已用时间，日志默认收起。
- 任务在后台运行；完成时正在查看任务页会自动进入阅读，在其他页面时不会打断当前操作。
- 课程库显示缩略图与内容数量；阅读页提供文稿、截图、文件分区，复制后明确反馈。
- 设置按通用、语音识别、AI 整理、运行环境、关于分组，有效输入防抖自动保存；无效输入保留并提示，不覆盖上次有效配置。默认设置供下一次任务使用，单次转换选项独立。
- 默认自定义标题栏，可在设置中改用系统标题栏（重启生效）；减少动态效果选项即时应用。

## 界面图标

内嵌 Google Material Icons Rounded SVG，运行时无需网络或安装图标字体。
导航图标统一为 20px，任务、刷新和加载使用各自对应的图标；共享组件中的
复选框、展开箭头、密码可见性和窗口控制也使用同套资源。
来源、版本和 Apache-2.0 许可证见 [图标资源](assets/material/README.md)。

设置与首次引导的独立子代理实机复审见 [验收记录](../docs/DESKTOP-SETTINGS-REVIEW.md)。关于页及应用菜单可查看版本与构建提交。
