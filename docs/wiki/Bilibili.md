# Bilibili login and video quality

[Wiki](Home.md) · [GUI](GUI.md) · [CLI](CLI.md) · [中文](Bilibili.zh.md)

## Bilibili Login and Video Quality

In the desktop app, open Settings → Connected accounts to scan a login QR code. Pasting a Bilibili link also shows an account entry. Public courses can be added without signing in.

```bash
course2md --login bilibili  # Scan and confirm with the Bilibili mobile app
course2md https://www.bilibili.com/video/BV1pb8o6yE8f --max-height 1080
course2md --logout bilibili # Remove the locally saved session
```

The CLI and desktop preview automatically reuse the session for metadata, subtitles, and downloads. Use `course2md --login bilibili <video URL>` to log in and then process a video. The default height limit is 1080p; use `--max-height 2160` for higher resolutions when available. Quality depends on the source and your account's existing permissions.

Credentials are stored separately from course output at `auth/bilibili.cookies.txt` inside the configuration directory (0600 permissions on Unix). Do not share this file. Each yt-dlp process uses a private temporary copy to avoid concurrent writes. Log in again when the session expires. Logout removes the local session; it does not revoke downloads already running.

Login may reduce Bilibili HTTP 412 rejections, but cannot guarantee their removal. Preview, metadata, and subtitle extraction retry these errors twice with a delay. Persistent failures show guidance to log in again, update yt-dlp, or retry later.
