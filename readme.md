# course2md

Turn **YouTube, Bilibili, or local videos** into illustrated Markdown / HTML notes. Add a video in the desktop app, generate notes, then organize and read them in your course library.

**English** · [中文](readme.zh.md) · [GitHub Wiki](https://github.com/mizorewww/course2md/wiki)

**Try 2.0 RC1:** [2.0.0-rc.1 release notes and installation](https://github.com/mizorewww/course2md/releases/tag/v2.0.0-rc.1). Use the `course2md-gui@rc` Homebrew cask; the default installation options below provide the stable release.

## Install the desktop app (GUI)

Choose one installation method for your system below. **The GUI includes its conversion engine; no separate CLI install is needed.**

### macOS: Apple Silicon, macOS 15 or later

**If you have Homebrew:** run this in Terminal to install the app and the video tools `ffmpeg` and `yt-dlp`:

```sh
brew install --cask mizorewww/tap/course2md-gui
```

After installation, open **course2md** from **Finder → Applications**.

**Manual installation:** [download the macOS installer (DMG)](https://github.com/mizorewww/course2md/releases/latest/download/course2md-gui-macos-arm64.dmg), open it, and drag **course2md.app** to Applications. This installs only the app; install video tools separately using the [macOS installation guide](https://github.com/mizorewww/course2md/wiki/GUI-Installation#macos-apple-silicon).

### Arch Linux / CachyOS: x86_64

If you have `yay`, run this to install the desktop app, video tools and required graphics libraries:

```sh
yay -S course2md-gui-bin
```

With `paru`, run `paru -S course2md-gui-bin`. Open **course2md** from your applications menu. Your graphics hardware needs a working Vulkan driver; see the [Linux installation guide](https://github.com/mizorewww/course2md/wiki/GUI-Installation#arch-linux--cachyos).

### Windows: x64

1. [Download the Windows portable package (ZIP)](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-windows-AMD64.zip) and **extract everything** to a permanent folder.
2. Install video tools in PowerShell:

   ```powershell
   winget install --id Gyan.FFmpeg -e
   winget install --id yt-dlp.yt-dlp -e
   ```

3. Open **course2md-desktop.exe** from the extracted folder. Keep `course2md.exe` beside it: this is the bundled engine. If the app was already open, quit and reopen it after installing tools.

### Ubuntu / other Linux distributions: x86_64

[Download the Linux desktop package (tar.gz)](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-linux-x86_64.tar.gz), install video tools and graphics libraries using the [Linux installation guide](https://github.com/mizorewww/course2md/wiki/GUI-Installation#other-linux-x64-distributions), then extract everything. Enter the extracted folder and run `./course2md-desktop`. Keep the `course2md` engine in the same folder.

The prebuilt package is based on Ubuntu 24.04. **Intel Mac and Linux ARM64 do not yet have prebuilt GUIs**; use the [CLI version](https://github.com/mizorewww/course2md/wiki/CLI-Guide). GitHub's **Source code** archives are developer sources, not app installers.

## First use

1. Open **course2md** and follow the setup guide to choose a notes folder and speech recognition method.
2. Check **Settings → Runtime environment**. `ffmpeg` (including `ffprobe`) processes video; `yt-dlp` handles online videos.
3. Add a video URL or local file and start conversion. Open the finished notes in your library.

Local recognition downloads models on first use. GPU / CPU recognition also needs `llama-server`; cloud recognition requires your own API configuration. The app provides setup controls; see the [first-launch guide](https://github.com/mizorewww/course2md/wiki/GUI-Installation#first-launch) for details.

## Documentation and help

For terminals and scripts, see the [standalone CLI installation and usage guide](https://github.com/mizorewww/course2md/wiki/CLI-Guide), which covers CLI installation, commands and options.

[All documentation (GitHub Wiki)](https://github.com/mizorewww/course2md/wiki) · [Upgrade and uninstall](https://github.com/mizorewww/course2md/wiki/GUI-Installation#upgrade-and-uninstall) · [Troubleshooting](https://github.com/mizorewww/course2md/wiki/Troubleshooting) · [Report a problem](https://github.com/mizorewww/course2md/issues)

[Development guide](desktop/README.md) · [Changelog](CHANGELOG.md) · [MIT license](LICENSE)
