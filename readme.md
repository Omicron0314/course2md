# course2md

Turn YouTube, Bilibili, or local videos into illustrated Markdown / HTML notes. Convert, organize, and read your courses in a native desktop app.

**English** · [中文](readme.zh.md) · [Documentation wiki](https://github.com/mizorewww/course2md/wiki)

## Install the desktop app

### macOS (Apple Silicon, macOS 15+)

```sh
brew install --cask mizorewww/tap/course2md-gui
```

Video tools are installed automatically. Or [download the DMG](https://github.com/mizorewww/course2md/releases/latest/download/course2md-gui-macos-arm64.dmg), open it, and drag the app to Applications.

### Arch Linux / CachyOS (x86_64)

```sh
yay -S course2md-gui-bin
```

Open **course2md** from your applications menu. You can also use `paru -S course2md-gui-bin`.

### Windows / other Linux distributions

| System | Download | Launch |
| --- | --- | --- |
| Windows x64 | [Portable ZIP](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-windows-AMD64.zip) | Extract everything, open `course2md-desktop.exe` |
| Linux x64 | [tar.gz](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-linux-x86_64.tar.gz) | Extract everything, run `./course2md-desktop` |

Manual downloads need separate video tools; see the [GUI installation guide](https://github.com/mizorewww/course2md/wiki/GUI-Installation). Keep the bundled files together.

## Get started

Open the app, choose a notes folder and recognition method in the setup guide, then add a video URL or local file and generate notes. The GUI includes its conversion engine; no separate CLI install is needed. Local recognition downloads models on first use.

[GUI usage and upgrades](https://github.com/mizorewww/course2md/wiki/GUI-Installation) · [Standalone CLI guide](https://github.com/mizorewww/course2md/wiki/CLI-Guide) · [Troubleshooting](https://github.com/mizorewww/course2md/wiki/Troubleshooting)

## Help and contributing

Report problems and suggestions in [Issues](https://github.com/mizorewww/course2md/issues). See the [development guide](desktop/README.md) to contribute. [Changelog](CHANGELOG.md) · [MIT license](LICENSE)
