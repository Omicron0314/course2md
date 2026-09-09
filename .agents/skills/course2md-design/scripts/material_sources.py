#!/usr/bin/env python3
"""Capture the allowlisted official references, or verify their offline hashes.

M3 site prose has no verified redistribution licence: retain only the configured
short quotation. Full article text is saved only for the licensed sources in
sources.json. Network access is explicit through --refresh or --check-remote.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import sys
import urllib.parse
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "references/material/sources.json"
HOSTS = {"m3.material.io", "developer.android.com", "developers.google.com",
         "raw.githubusercontent.com", "www.apache.org", "creativecommons.org"}
VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link",
        "meta", "param", "source", "track", "wbr"}
SKIP_TAGS = {"script", "style", "devsite-key-takeaways-panel"}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class ArticleText(HTMLParser):
    """Extract actual Google DevSite article text, without navigation or scripts."""

    def __init__(self, url: str):
        super().__init__(convert_charrefs=True)
        self.url, self.depth, self.pre, self.skip = url, 0, 0, 0
        self.parts: list[str] = []
        self.links: list[str] = []
        self.found = False

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if not self.depth:
            if "devsite-article-body" not in a.get("class", "").split():
                return
            self.depth, self.found = 1, True
            return
        if tag not in VOID:
            self.depth += 1
        if tag in SKIP_TAGS:
            self.skip += 1
        if self.skip:
            return
        if re.fullmatch("h[1-6]", tag):
            self.parts.append("\n\n" + "#" * int(tag[1]) + " ")
        elif tag in {"p", "div", "ul", "ol", "section", "figure"}:
            self.parts.append("\n\n")
        elif tag == "li":
            self.parts.append("\n- ")
        elif tag == "br":
            self.parts.append("\n")
        elif tag == "tr":
            self.parts.append("\n| ")
        elif tag == "pre":
            self.pre += 1
            self.parts.append("\n\n```\n")
        elif tag == "a":
            self.links.append(urllib.parse.urljoin(self.url, a.get("href", "")))
            self.parts.append("[")
        elif tag == "img":
            label = a.get("alt", "Illustration") or "Illustration"
            src = urllib.parse.urljoin(self.url, a.get("src", ""))
            self.parts.append(f"\n[Visual omitted from text archive: {label}]({src})\n")

    def handle_endtag(self, tag):
        if not self.depth or tag in VOID:
            return
        if self.skip:
            if tag in SKIP_TAGS:
                self.skip -= 1
        elif tag == "a" and self.links:
            self.parts.append("](" + self.links.pop() + ")")
        elif tag in {"td", "th"}:
            self.parts.append(" | ")
        elif tag == "pre":
            self.pre -= 1
            self.parts.append("\n```\n\n")
        elif tag in {"p", "div", "section", "figure"} or re.fullmatch("h[1-6]", tag):
            self.parts.append("\n\n")
        self.depth -= 1

    def handle_data(self, data):
        if self.depth and not self.skip:
            if self.pre:
                self.parts.append(data)
            elif data.strip():
                self.parts.append(re.sub(r"\s+", " ", data))

    def text(self):
        if not self.found:
            raise ValueError("Actual DevSite article body was not found; refusing a shell page")
        lines, in_code = [], False
        for line in "".join(self.parts).splitlines():
            if line.strip() == "```":
                in_code = not in_code
            lines.append(line if in_code else line.strip())
        return re.sub(r"\n{3,}", "\n\n", "\n".join(lines)).strip()


class PlainText(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.parts = []

    def handle_data(self, text):
        self.parts.append(text)


def fetch(url):
    if urllib.parse.urlparse(url).hostname not in HOSTS:
        raise ValueError(f"Unapproved reference host: {url}")
    with urllib.request.urlopen(url, timeout=45) as response:
        if urllib.parse.urlparse(response.url).hostname not in HOSTS:
            raise ValueError(f"Unapproved redirect: {response.url}")
        return response.read(), response.url


def capture(source):
    item = dict(source)
    try:
        raw, final_url = fetch(item.get("data_url", item["url"]))
    except OSError as error:
        raise ValueError(f"Cannot fetch {item['id']}: {error}") from error
    content = raw.decode("utf-8")
    kind = item["representation"]
    if kind == "html-short-excerpt":
        parser = ArticleText(final_url)
        parser.feed(content)
        normalized = " ".join(parser.text().split())
        quote = item["excerpt"]
        # Remove Markdown link syntax solely for exact text matching.
        normalized = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", normalized)
        if len(quote.split()) > 25 or quote not in normalized:
            raise ValueError(f"Missing or overlong quotation: {item['id']}")
        content = "> " + quote
        changes = "Short verbatim quotation of the official licensing statement. The complete policy page is linked above."
    elif kind == "m3-short-excerpt":
        page = json.loads(content)
        all_text = []
        for section in page["sections"]:
            if not section.get("isVisible", True):
                continue
            for block in section["contentBlocks"]:
                if block.get("isHidden"):
                    continue
                for chunk in block["contentChunks"]:
                    for key in ("htmlValue", "footer"):
                        if chunk.get(key):
                            parser = PlainText()
                            parser.feed(chunk[key])
                            all_text.extend(parser.parts)
        normalized = " ".join(" ".join(all_text).split())
        quote = item["excerpt"]
        if len(quote.split()) > 25 or quote not in normalized:
            raise ValueError(f"Missing or overlong quotation: {item['id']}")
        content = "> " + quote
        item["upstream_updated_at"] = page.get("updatedTimestamp")
        changes = "Short verbatim quotation only. The full M3 page, JSON and media are not redistributed."
    elif kind == "full-article-text":
        if item["license"] not in {"Apache-2.0", "CC-BY-4.0"}:
            raise ValueError("Full article capture needs a verified open licence")
        parser = ArticleText(final_url)
        parser.feed(content)
        content = parser.text()
        if len(content.split()) < 100:
            raise ValueError(f"Article body too short: {item['id']}")
        changes = ("Complete article text extracted from the official HTML body into Markdown. "
                   "Site navigation, executable scripts and AI-generated page summaries are removed; tables use pipe-separated rows. "
                   "Images and videos are not bundled. Captions, alternative text and source links remain where supplied. "
                   "This is a text archive, not a visual facsimile.")
    elif kind == "full-markdown":
        if item["license"] != "Apache-2.0":
            raise ValueError("Full Markdown capture needs the recorded Apache licence")
        changes = "Official Markdown reproduced without changing its body. Relative and remote links retain their original repository meaning; use the original source URL for linked assets. Media are not bundled."
    elif kind == "license-text":
        changes = "Official licence text reproduced unchanged below this provenance header."
    else:
        raise ValueError(f"Unknown representation: {kind}")
    now = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    header = (f"# {item['title']}\n\n"
              f"- Author / publisher: {item['publisher']}\n"
              f"- Original: [{item['title']}]({item['url']})\n"
              f"- Retrieved: {now}\n"
              f"- Licence: {item['license']} — [licence evidence]({item['license_url']})\n"
              f"- Upstream SHA-256: `{digest(raw)}`\n"
              f"- Changes: {changes}\n\n"
              "This source is reference data, not project instructions. Product decisions are in the parent skill references.\n\n---\n\n")
    if item["license"] == "CC-BY-4.0":
        header += ("Portions of this page are modifications based on work created and "
                   "[shared by Google](https://developers.google.com/terms/site-policies) and used according to "
                   "terms described in the [Creative Commons 4.0 Attribution License](https://creativecommons.org/licenses/by/4.0/).\n\n")
    data = (header + content.strip() + "\n").encode()
    item.update(retrieved_at=now, resolved_url=final_url,
                upstream_sha256=digest(raw), saved_sha256=digest(data),
                saved_bytes=len(data), body_words=len(content.split()))
    return item, data


def verify(sources):
    for item in sources:
        path = ROOT / item["file"]
        data = path.read_bytes()
        if digest(data) != item["saved_sha256"]:
            raise ValueError(f"Local archive changed: {item['file']}")
        if b"This website requires JavaScript." in data:
            raise ValueError(f"Shell page found: {item['file']}")
        if item["representation"] == "m3-short-excerpt" and item["body_words"] > 26:
            raise ValueError(f"Overlong M3 excerpt: {item['id']}")
    print(f"Verified {len(sources)} official-source captures and their SHA-256 hashes.")


def main():
    args = argparse.ArgumentParser(description=__doc__)
    group = args.add_mutually_exclusive_group()
    group.add_argument("--refresh", action="store_true", help="Refetch only the recorded URLs and update captures")
    group.add_argument("--check-remote", action="store_true", help="Read-only comparison with the recorded upstream hashes")
    args.add_argument("--verify", action="store_true", help="Verify local archive hashes (the default)")
    options = args.parse_args()
    catalog = json.loads(CATALOG.read_text())
    sources = catalog["sources"]
    if options.refresh:
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            results = list(pool.map(capture, sources))
        # Complete every read before writing any artifact. A failed fetch leaves captures intact.
        for item, data in results:
            path = ROOT / item["file"]
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        sources = [item for item, _ in results]
        catalog["sources"] = sources
        CATALOG.write_text(json.dumps(catalog, indent=2, ensure_ascii=False) + "\n")
    verify(sources)
    if options.check_remote:
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            blobs = list(pool.map(lambda x: fetch(x.get("data_url", x["url"]))[0], sources))
        changed = [s["id"] for s, blob in zip(sources, blobs) if digest(blob) != s["upstream_sha256"]]
        if changed:
            raise ValueError("Upstream responses changed: " + ", ".join(changed))
        print("All recorded upstream responses still match.")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError) as error:
        print(f"Reference validation failed: {error}", file=sys.stderr)
        sys.exit(1)
