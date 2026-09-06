# GUI 安装与使用

[Wiki](Home.zh.md) · [CLI](CLI.zh.md) · [English](GUI.md)

## 安装

### macOS Apple Silicon

需要 macOS 15+。推荐 Homebrew，自动安装应用及 ffmpeg、yt-dlp：

```sh
brew install --cask mizorewww/tap/course2md-gui
```

也可从 [Releases](https://github.com/mizorewww/course2md/releases/latest) 下载 `course2md-gui-macos-arm64.dmg`，打开后将 course2md.app 拖入「应用程序」。手动安装还需 `brew install ffmpeg yt-dlp`。正式应用与 DMG 经 Developer ID 签名和 Apple 公证。

当前没有 Intel Mac 预编译 GUI；可使用 [CLI](CLI.zh.md)。

### Arch Linux / CachyOS

x86_64 用户安装 GUI 包，视频与图形运行库由包管理器处理：

```sh
yay -S course2md-gui-bin
# 或 paru -S course2md-gui-bin
```

从应用菜单打开 course2md，或运行 `course2md-desktop`。需要适合显卡的 Vulkan 驱动；无硬件驱动时可尝试 `vulkan-swrast` 软件渲染。

`course2md-gui-bin` 提供窗口和私有转换引擎；`course2md-bin` 提供终端命令，两者可以共存。

### Windows x64

1. 下载 [GUI ZIP](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-windows-AMD64.zip)，完整解压至固定目录。
2. 在 PowerShell 安装视频工具：

   ```powershell
   winget install --id Gyan.FFmpeg -e
   winget install --id yt-dlp.yt-dlp -e
   ```

3. 重新打开终端或应用，启动解压目录中的 `course2md-desktop.exe`，保留旁边的 `course2md.exe`。

### 其他 Linux x64

下载 [GUI tar.gz](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-linux-x86_64.tar.gz)，完整解压，进入目录运行 `./course2md-desktop`。保留旁边的 `course2md`。

Ubuntu 24.04 可先安装：

```sh
sudo apt update
sudo apt install ffmpeg yt-dlp libxcb1 libxkbcommon0 libxkbcommon-x11-0 libfontconfig1 libx11-6 libwayland-client0 libvulkan1 mesa-vulkan-drivers
```

预编译包基于 Ubuntu 24.04；旧发行版可能因 glibc 版本而无法运行。当前没有 Linux ARM64 GUI，可使用 [CLI](CLI.zh.md)。

## 首次启动

1. 选择保存笔记的目录和识别方式，缺少工具时进入安装帮助。
2. 在「设置 → 运行环境」确认工具可用；安装外部工具后需重启应用。
3. 添加在线视频链接或本地视频，确认来源和导出格式，再生成笔记。
4. 在课程库阅读文稿、截图与文件；可用列表、卡片、文件夹分组整理。

有可用字幕时默认优先使用字幕。Apple 原生识别首次下载模型；本地 GPU/CPU 识别还需 llama-server 与模型；云端 API 需自己的端点、密钥和模型。详见[识别与模型](ASR.zh.md)、[云端与 AI](AI.zh.md)。

Bilibili 可在「设置 → 连接账号」扫码登录，或在来源页进入登录。详见[Bilibili 指南](Bilibili.zh.md)。

GUI 和 CLI 共用配置；GUI 的有效设置自动保存。逻辑文件夹的移动或删除不会删除笔记文件。任务取消、运行状态、错误详情均可在应用内查看。

## 升级与卸载

Homebrew：`brew update && brew upgrade --cask course2md-gui`；AUR：`yay -Syu` 或 `paru -Syu`。

手动安装：退出旧应用，再替换整个应用或完整解压的新目录，避免混用新旧引擎。保留配置、课程输出和模型缓存；独立 CLI 单独升级。

Homebrew 卸载用 `brew uninstall --cask course2md-gui`；AUR 用 `sudo pacman -R course2md-gui-bin`。手动安装删除应用目录即可。以上操作不要求删除个人笔记、配置或模型缓存。

## 常见问题

- GUI 已有引擎，但不会自动安装终端 `course2md` 命令。需要脚本运行时另外安装 [CLI](CLI.zh.md)。
- 不要把 GitHub 的 Source code 压缩包当作应用安装包。
- 缺少工具、模型下载失败或图形启动问题，见[故障排查](Troubleshooting.zh.md)。
