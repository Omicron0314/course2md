#!/usr/bin/env python3
"""Resolve a published version for manual, tag, or workflow-dispatch releases."""
import json
import os
import re
import urllib.request


def api(path):
    request = urllib.request.Request(
        f"https://api.github.com/repos/{os.environ['GITHUB_REPOSITORY']}/{path}",
        headers={"Authorization": f"Bearer {os.environ['GH_TOKEN']}",
                 "Accept": "application/vnd.github+json"},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def main():
    version = os.environ.get("REQUESTED_VERSION", "")
    if not version:
        branch = os.environ.get("RELEASE_BRANCH", "")
        if re.fullmatch(r"v\d+\.\d+\.\d+", branch):
            version = branch
        else:
            sha = os.environ.get("RELEASE_SHA", "")
            releases = api("releases?per_page=100")
            matches = [r for r in releases if sha and r["target_commitish"] == sha
                       and not r["draft"] and not r["prerelease"]]
            if len(matches) != 1:
                raise SystemExit("Cannot identify one published release for the triggering commit")
            version = matches[0]["tag_name"]
    version = version.removeprefix("v")
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise SystemExit("Expected a stable version such as 1.7.0")
    release = api(f"releases/tags/v{version}")
    if release["draft"] or release["prerelease"]:
        raise SystemExit("Package channels only publish stable releases")
    with open(os.environ["GITHUB_OUTPUT"], "a") as output:
        output.write(f"version={version}\n")
    print(f"Updating GUI and CLI packages to {version}")


if __name__ == "__main__":
    main()
