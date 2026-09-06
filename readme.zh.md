# course2md

把 YouTube、Bilibili 或本地视频转换成带截图的 Markdown / HTML 笔记，用桌面应用完成转换、整理和阅读。

[English](readme.md) · **中文** · [文档 Wiki](docs/wiki/Home.zh.md)

## 安装桌面应用

### macOS（Apple Silicon，macOS 15+）

```sh
brew install --cask mizorewww/tap/course2md-gui
```

自动安装视频工具。也可[下载 DMG](https://github.com/mizorewww/course2md/releases/latest/download/course2md-gui-macos-arm64.dmg)，打开后拖入「应用程序」。

### Arch Linux / CachyOS（x86_64）

```sh
yay -S course2md-gui-bin
```

安装后从应用菜单打开 **course2md**。也可用 `paru -S course2md-gui-bin`。

### Windows / 其他 Linux

| 系统 | 下载 | 启动 |
| --- | --- | --- |
| Windows x64 | [便携 ZIP](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-windows-AMD64.zip) | 完整解压，打开 `course2md-desktop.exe` |
| Linux x64 | [tar.gz](https://github.com/mizorewww/course2md/releases/latest/download/course2md-desktop-linux-x86_64.tar.gz) | 完整解压，运行 `./course2md-desktop` |

手动下载的包需另装视频工具，见 [GUI 安装指南](docs/wiki/GUI.zh.md)。请保留包内配套文件。

## 开始使用

打开应用，按向导选择笔记目录和识别方式，然后添加视频链接或本地文件并生成笔记。GUI 已包含转换引擎，无需再安装 CLI。首次本地识别需要下载模型。

[GUI 使用与升级](docs/wiki/GUI.zh.md) · [独立 CLI 指南](docs/wiki/CLI.zh.md) · [故障排查](docs/wiki/Troubleshooting.zh.md)

## 帮助与贡献

问题和建议请提交 [Issue](https://github.com/mizorewww/course2md/issues)。开发与贡献见 [Wiki](docs/wiki/Home.zh.md#开发与项目记录)。[更新记录](CHANGELOG.md) · [MIT 许可证](LICENSE)
