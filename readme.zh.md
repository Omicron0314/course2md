# course2md

把 **YouTube、Bilibili 或本地视频**转换成带截图的 Markdown / HTML 笔记。在桌面应用里添加视频、生成笔记，再用课程库整理和阅读。

[English](readme.md) · **中文** · [GitHub Wiki](https://github.com/mizorewww/course2md/wiki)

**2.0 alpha 试用：**[2.0.0-alpha.1 发布说明与安装](https://github.com/mizorewww/course2md/releases/tag/v2.0.0-alpha.1)。Homebrew 使用 `course2md-gui@alpha`；以下默认安装入口继续提供稳定版。

## 安装桌面应用（GUI）

按你的系统选择下面的一种安装方式。**GUI 已自带转换引擎，无需另装 CLI。**

### macOS：M 系列芯片，macOS 15 及以上

**已安装 Homebrew：**在终端运行以下命令，它会安装应用及视频工具 `ffmpeg`、`yt-dlp`：

```sh
brew install --cask mizorewww/tap/course2md-gui
```

装完后，从「访达 → 应用程序」打开 **course2md**。

**手动安装：**[下载 macOS 安装包（DMG）](https://github.com/mizorewww/course2md/releases/latest/download/course2md-gui-macos-arm64.dmg)，打开后将 **course2md.app** 拖入「应用程序」。这种方式只安装应用，视频工具需按 [macOS 安装指南](https://github.com/mizorewww/course2md/wiki/%E5%AE%89%E8%A3%85%E6%A1%8C%E9%9D%A2%E5%BA%94%E7%94%A8#macos-apple-silicon)另行安装。

### Arch Linux / CachyOS：x86_64

已安装 `yay` 的用户运行以下命令；它会安装桌面应用、视频工具和所需图形库：

```sh
yay -S course2md-gui-bin
```

使用 `paru` 时运行 `paru -S course2md-gui-bin`。装完后从应用菜单打开 **course2md**。显卡需有可用的 Vulkan 驱动，详见 [Linux 安装指南](https://github.com/mizorewww/course2md/wiki/%E5%AE%89%E8%A3%85%E6%A1%8C%E9%9D%A2%E5%BA%94%E7%94%A8#arch-linux--cachyos)。

### Windows：x64

1. [下载 Windows 便携包（ZIP）](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-windows-AMD64.zip)，**完整解压**到固定文件夹。
2. 在 PowerShell 中安装视频工具：

   ```powershell
   winget install --id Gyan.FFmpeg -e
   winget install --id yt-dlp.yt-dlp -e
   ```

3. 打开解压目录中的 **course2md-desktop.exe**。旁边的 `course2md.exe` 是配套引擎，请保留在同一目录。如果应用已打开，安装工具后退出并重新打开。

### Ubuntu / 其他 Linux：x86_64

[下载 Linux 桌面包（tar.gz）](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-linux-x86_64.tar.gz)，按 [Linux 安装指南](https://github.com/mizorewww/course2md/wiki/%E5%AE%89%E8%A3%85%E6%A1%8C%E9%9D%A2%E5%BA%94%E7%94%A8#其他-linux-x64)安装视频工具和图形运行库，再完整解压。进入解压目录，运行 `./course2md-desktop`，保留同目录的 `course2md` 引擎。

预编译包基于 Ubuntu 24.04。**Intel Mac 和 Linux ARM64 暂无预编译 GUI**，可使用 [CLI 版本](https://github.com/mizorewww/course2md/wiki/CLI-%E6%8C%87%E5%8D%97)。GitHub 的 **Source code** 压缩包是开发源码，不是应用安装包。

## 第一次使用

1. 打开 **course2md**，按向导选择笔记保存目录和语音识别方式。
2. 在「设置 → 运行环境」确认工具可用。`ffmpeg`（包含 `ffprobe`）用于处理视频；`yt-dlp` 用于在线视频。
3. 添加视频链接或本地文件，开始生成，完成后在课程库打开笔记。

本地识别首次使用需要下载模型；选择 GPU / CPU 识别还需安装 `llama-server`；选择云端识别需要填写自己的 API 配置。应用提供设置入口，详细步骤见 [首次使用指南](https://github.com/mizorewww/course2md/wiki/%E5%AE%89%E8%A3%85%E6%A1%8C%E9%9D%A2%E5%BA%94%E7%94%A8#首次启动)。

## 文档与帮助

终端和脚本用户请看 [独立 CLI 安装与使用指南](https://github.com/mizorewww/course2md/wiki/CLI-%E6%8C%87%E5%8D%97)。CLI 的安装步骤、命令和参数都在该指南中。

[全部文档（GitHub Wiki）](https://github.com/mizorewww/course2md/wiki) · [升级与卸载](https://github.com/mizorewww/course2md/wiki/%E5%AE%89%E8%A3%85%E6%A1%8C%E9%9D%A2%E5%BA%94%E7%94%A8#升级与卸载) · [故障排查](https://github.com/mizorewww/course2md/wiki/%E6%95%85%E9%9A%9C%E6%8E%92%E6%9F%A5) · [反馈问题](https://github.com/mizorewww/course2md/issues)

[开发指南](desktop/README.md) · [更新记录](CHANGELOG.md) · [MIT 许可证](LICENSE)
