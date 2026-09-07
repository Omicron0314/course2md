#!/usr/bin/env python3
"""Isolated synthetic media and a controllable local service for desktop UX verification."""
import argparse
import json
import plistlib
import re
import shutil
import socket
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


def bundle(root, label, window_size):
    """Package already-built native binaries; isolate configuration and synthetic sources."""
    project = Path(__file__).resolve().parents[2]
    app = root / f"course2md Validation {label}.app"
    binaries = app / "Contents/MacOS"
    binaries.mkdir(parents=True, exist_ok=True)
    for source in [project / "target/debug/course2md", project / "desktop/target/debug/course2md-desktop", project / "target/debug/mlx.metallib"]:
        if source.is_file():
            shutil.copy2(source, binaries / source.name)
        elif source.suffix != ".metallib":
            raise RuntimeError(f"Build this executable first: {source}")
    testbin = root / "bin"
    testbin.mkdir(exist_ok=True)
    extractor = testbin / "yt-dlp"
    shutil.copy2(project / "desktop/scripts/ux_ytdlp.py", extractor)
    extractor.chmod(0o755)
    frame_tool = testbin / "ffmpeg"
    shutil.copy2(project / "desktop/scripts/ux_ffmpeg.py", frame_tool)
    frame_tool.chmod(0o755)
    config = root / "config/course2md"
    config.mkdir(parents=True, exist_ok=True)
    config_file = config / "config.toml"
    if not config_file.exists():
        config_file.write_text('[defaults]\nout = ' + json.dumps(str(root / "library")) + '\n')
    for directory in ["library", "screenshots", "exports"]:
        (root / directory).mkdir(exist_ok=True)
    resources = app / "Contents/Resources"
    resources.mkdir(exist_ok=True)
    shutil.copy2(project / "desktop/assets/icon.icns", resources / "course2md.icns")
    with (app / "Contents/Info.plist").open("wb") as stream:
        plistlib.dump({
            "CFBundleExecutable": "course2md-desktop", "CFBundleIdentifier": "dev.course2md.ux-validation." + label,
            "CFBundleName": "course2md Validation " + label, "CFBundlePackageType": "APPL",
            "CFBundleVersion": "1", "CFBundleIconFile": "course2md.icns",
            "NSHighResolutionCapable": True, "NSPrincipalClass": "NSApplication",
            "LSEnvironment": {
                "XDG_CONFIG_HOME": str(root / "config"), "COURSE2MD_BIN": str(binaries / "course2md"),
                "XDG_CACHE_HOME": str(root / "cache"),
                "COURSE2MD_UX_FIXTURE_ROOT": str(root), "DYLD_LIBRARY_PATH": "/usr/lib/swift",
                "QWEN3_CACHE_DIR": str(root / "model-cache/apple"),
                "HF_HOME": str(root / "model-cache/huggingface"),
                "COURSE2MD_VALIDATION_WINDOW": window_size,
                "COURSE2MD_VALIDATION_PANIC_LOG": str(root / f"{label}-panic.log"),
                "COURSE2MD_TOOL_PATH": str(testbin) + ":/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
                "PATH": str(testbin) + ":/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
            },
        }, stream)
    print(json.dumps({"app": str(app), "synthetic_sources": "BV1UX* only"}))


def prepare(root):
    media = root / "media"
    media.mkdir(parents=True, exist_ok=True)
    voice = media / "speech.aiff"
    if not voice.exists():
        subprocess.run(["say", "-o", str(voice), "The blue notebook contains seven pages. Risk and return are related. A diversified portfolio can reduce some risks."], check=True)
    for name, color, title in [("lecture-a", "blue", "课程 A：风险与收益"), ("lecture-b", "green", "课程 B：投资组合"), ("no-subtitles", "purple", "待识别的讲解"), ("lecture-external", "red", "本地源 D：外语课程")]:
        target = media / f"{name}.mp4"
        if not target.exists():
            subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", f"color=c={color}:s=640x360:r=2", "-i", str(voice), "-t", "18", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-metadata", f"title={title}", str(target)], check=True)
        if name in ("lecture-a", "lecture-b"):
            lines = []
            for i in range(18):
                lines.append(f"{i+1}\n00:00:{i:02},000 --> 00:00:{i:02},900\n第 {i+1} 段：{title}。收益需要结合风险判断，分散投资可以减少个别资产带来的影响。保留课程原意和时间关系。\n")
            (media / f"{name}.srt").write_text("\n".join(lines))
    (media / "different-name.fr.srt").write_text("1\n00:00:00,000 --> 00:00:07,000\nLe risque et le rendement sont liés.\n\n2\n00:00:07,000 --> 00:00:17,000\nLa diversification peut réduire certains risques.\n")
    (media / "invalid-subtitles.srt").write_text("This file intentionally contains no valid subtitle cues.\n")
    (root / "service-mode.json").write_text(json.dumps({"mode": "valid"}))
    print(json.dumps({"media": str(media), "service_mode": str(root / "service-mode.json")}, ensure_ascii=False))


def serve(root, port):
    lock = threading.Lock()

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_GET(self):
            self.answer(200, {"fixture": "course2md UX verification", "synthetic": True})

        def answer(self, status, data):
            body = json.dumps(data, ensure_ascii=False).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            try:
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def do_POST(self):
            raw = self.rfile.read(min(int(self.headers.get("Content-Length", 0)), 32 * 1024 * 1024))
            try:
                body = json.loads(raw)
            except (ValueError, UnicodeDecodeError):
                body = {}
            try:
                policy = json.loads((root / "service-mode.json").read_text())
            except (OSError, ValueError):
                policy = {"mode": "valid"}
            text = json.dumps(body, ensure_ascii=False)
            mode = policy.get("mode", "valid")
            with lock, (root / "service-requests.jsonl").open("a") as log:
                purpose = "speech" if "input_audio" in text or "audio/transcriptions" in self.path else "summary" if "tldr" in text or "photosynthesis" in text else "proofread"
                log.write(json.dumps({"time": time.time(), "path": self.path, "mode": mode, "model": body.get("model"), "purpose": purpose, "bytes": len(raw), "has_audio": "input_audio" in text or "audio/transcriptions" in self.path, "has_images": "image_url" in text, "has_authorization": bool(self.headers.get("Authorization")), "has_test_rule": "UX-RULE-42" in text}, ensure_ascii=False) + "\n")
            if mode == "drop":
                self.connection.shutdown(socket.SHUT_RDWR)
                self.connection.close()
                return
            if mode == "slow":
                time.sleep(policy.get("seconds", 25))
            if mode in ("unauthorized", "rate-limit", "server-error"):
                self.answer({"unauthorized": 401, "rate-limit": 429, "server-error": 503}[mode], {"error": {"message": "Synthetic service response"}})
                return
            if mode == "invalid-200":
                self.answer(200, {"status": "ok", "result": "wrong protocol"})
                return
            if "audio/transcriptions" in self.path:
                self.answer(200, {"text": "The blue notebook contains seven pages."})
                return
            if "input_audio" in text:
                content = "The blue notebook contains seven pages."
            elif "tldr" in text or "photosynthesis" in text:
                if mode == "summary-error":
                    self.answer(400, {"error": {"message": "Synthetic summary failure"}})
                    return
                if "photosynthesis" in text:
                    summary = "Photosynthesis uses sunlight and releases oxygen."
                else:
                    summary = "课程说明风险和收益的关系，并介绍分散投资。"
                content = json.dumps({"tldr": summary, "key_points": [summary], "outline": [{"t": 0, "title": "课程要点", "detail": summary}]}, ensure_ascii=False)
            elif "student have" in text:
                content = json.dumps({"segments": [{"id": 0, "text": "The student has three books."}, {"id": 1, "text": "Water freezes at zero degrees Celsius."}]})
            elif "card shown is red" in text:
                content = json.dumps({"segments": [{"id": 0, "text": "The card shown is blue."}]})
            else:
                segments = []
                for message in body.get("messages", []):
                    values = message.get("content", "")
                    values = [values] if isinstance(values, str) else [v.get("text", "") for v in values if isinstance(v, dict)]
                    for value in values:
                        try:
                            parsed = json.loads(value)
                            segments = parsed.get("segments", []) if isinstance(parsed, dict) else parsed
                        except (ValueError, TypeError):
                            found = re.findall(r'\{"id"\s*:\s*(\d+),\s*"text"\s*:\s*("(?:[^"\\]|\\.)*")\}', value)
                            segments.extend({"id": int(i), "text": json.loads(t)} for i, t in found)
                content = json.dumps({"segments": segments}, ensure_ascii=False)
            self.answer(200, {"choices": [{"message": {"content": content}}]})

    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    print(json.dumps({"port": server.server_address[1], "root": str(root)}), flush=True)
    server.serve_forever()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=["prepare", "serve", "bundle"])
    parser.add_argument("--root", type=Path, default=Path("/tmp/course2md-ux-validation"))
    parser.add_argument("--port", type=int, default=8769)
    parser.add_argument("--label", default="v2", help="Unique test bundle label to avoid cached launch environments")
    parser.add_argument("--window", default="1140x820", help="Initial native window dimensions for debug validation builds")
    args = parser.parse_args()
    args.root.mkdir(parents=True, exist_ok=True)
    if args.action == "prepare":
        prepare(args.root)
    elif args.action == "bundle":
        if not re.fullmatch(r"[a-zA-Z0-9-]+", args.label):
            parser.error("label must contain only letters, digits, or hyphens")
        if not re.fullmatch(r"[0-9]+x[0-9]+", args.window):
            parser.error("window must be WIDTHxHEIGHT")
        bundle(args.root, args.label, args.window)
    else:
        serve(args.root, args.port)
