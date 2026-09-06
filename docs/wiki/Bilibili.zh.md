# Bilibili 登录与清晰度

[Wiki](Home.zh.md) · [GUI](GUI.zh.md) · [CLI](CLI.zh.md) · [English](Bilibili.md)

## Bilibili 登录与清晰度

桌面端可在「设置 → 连接账号」扫码登录；粘贴 Bilibili 链接后也会显示账号入口。公开课程无需先登录。

```bash
course2md --login bilibili  # 用哔哩哔哩 App 扫码并确认
course2md https://www.bilibili.com/video/BV1pb8o6yE8f --max-height 1080
course2md --logout bilibili # 清除本地登录状态
```

登录后，CLI 与桌面客户端的预览、字幕、下载会自动复用登录状态。也可以用 `course2md --login bilibili <视频链接>` 在登录后直接处理视频。默认清晰度上限为 1080p；需要更高分辨率时指定 `--max-height 2160`。实际画质受视频源和账号权限限制，登录不会赋予大会员权限。

登录凭据保存在配置目录下的 `auth/bilibili.cookies.txt`，与课程输出分开存储（Unix 文件权限为 0600），请勿分享此文件。每个 yt-dlp 进程使用临时副本，避免并发写入破坏登录状态。凭据过期时重新执行登录命令；退出仅删除本机保存的凭据，不撤销已经启动的下载会话。

Bilibili HTTP 412 是平台请求限制，登录可能缓解但不能保证消除。预览、元数据和字幕提取遇到 412 会等待后最多重试两次；持续失败时提示重新登录、更新 yt-dlp 或稍后再试。
