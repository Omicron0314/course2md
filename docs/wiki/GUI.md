# GUI installation and usage

[Wiki](Home.md) · [CLI](CLI.md) · [中文](GUI.zh.md)

## Installation

### macOS Apple Silicon

Requires macOS 15+. Homebrew installs the app, ffmpeg and yt-dlp:

```sh
brew install --cask mizorewww/tap/course2md-gui
```

Alternatively, download the [DMG](https://github.com/mizorewww/course2md/releases/latest/download/course2md-gui-macos-arm64.dmg), open it and drag course2md.app to Applications. Manual installs also need `brew install ffmpeg yt-dlp`. Release apps and disk images are Developer ID signed and notarized by Apple.

There is no prebuilt Intel Mac GUI; use the [CLI](CLI.md).

### Arch Linux / CachyOS

For x86_64, install the GUI with its video tools and graphics libraries:

```sh
yay -S course2md-gui-bin
# Or: paru -S course2md-gui-bin
```

Open course2md from your applications menu, or run `course2md-desktop`. Install a Vulkan driver for your GPU; `vulkan-swrast` provides a software rendering option.

`course2md-gui-bin` provides the window and private engine; `course2md-bin` provides the terminal command. They can coexist.

### Windows x64

1. Download the [GUI ZIP](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-windows-AMD64.zip) and extract everything to a permanent folder.
2. Install video tools in PowerShell:

   ```powershell
   winget install --id Gyan.FFmpeg -e
   winget install --id yt-dlp.yt-dlp -e
   ```

3. Restart the app/terminal, then open `course2md-desktop.exe`. Keep `course2md.exe` beside it.

### Other Linux x64 distributions

Download and extract the [GUI tar.gz](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-linux-x86_64.tar.gz), enter its folder and run `./course2md-desktop`. Keep the bundled `course2md` beside it.

On Ubuntu 24.04, install:

```sh
sudo apt update
sudo apt install ffmpeg yt-dlp libxcb1 libxkbcommon0 libxkbcommon-x11-0 libfontconfig1 libx11-6 libwayland-client0 libvulkan1 mesa-vulkan-drivers
```

The archive is built on Ubuntu 24.04; older distributions may have incompatible glibc versions. Linux ARM64 currently has [CLI](CLI.md) builds only.

## First launch

1. Choose a notes folder and recognition method. Missing tools lead to installation help.
2. Check Settings → Runtime environment. Restart the app after installing tools.
3. Add a video URL or local file, check the source and export formats, then generate notes.
4. Read transcripts, screenshots and files in the library; organize courses using lists, cards and folder groups.

Available subtitles take priority by default. Apple-native recognition downloads models on first use. Local GPU/CPU recognition needs llama-server and models; cloud APIs need your endpoint, key and model. See [recognition and models](ASR.md) and [cloud and AI](AI.md).

Bilibili QR login is available in account settings and the video source page. See the [Bilibili guide](Bilibili.md).

GUI and CLI share configuration. Valid GUI settings save automatically. Moving or deleting logical folders does not delete notes. Task progress, cancellation and error details are available in the app.

## Upgrade and uninstall

Homebrew: `brew update && brew upgrade --cask course2md-gui`. AUR: `yay -Syu` or `paru -Syu`.

For manual installs, quit the old app and replace the whole app/archive. Do not mix engine versions. Preserve settings, notes and model caches. Upgrade standalone CLI separately.

Uninstall with `brew uninstall --cask course2md-gui` or `sudo pacman -R course2md-gui-bin`. For manual installs, remove the application folder. You do not need to delete personal notes, settings or model caches.

## Common questions

- The GUI bundles its engine but does not add a `course2md` terminal command. Install the [CLI](CLI.md) separately for scripting.
- GitHub's Source code archives are not application installers.
- For missing tools, failed model downloads or graphics startup problems, see [troubleshooting](Troubleshooting.md).
