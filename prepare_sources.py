"""Fetch pinned sources; refuse to overwrite an existing checkout."""
from pathlib import Path
import json
import subprocess

root = Path(__file__).resolve().parent
sources = json.loads((root / "sources.json").read_text(encoding="utf-8"))
for name, entry in sources["repositories"].items():
    dest = root / entry["destination"]
    if dest.exists():
        raise SystemExit(f"Refusing to replace {dest}; use a fresh export directory")
    dest.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["git", "init", str(dest)], check=True)
    def git(*args):
        subprocess.run(["git", "-C", str(dest), *args], check=True)
    git("remote", "add", "origin", entry["url"])
    git("fetch", "--depth", "1", "origin", entry["base"])
    git("checkout", "--detach", "FETCH_HEAD")
    if "patch" in entry and (root / entry["patch"]).stat().st_size:
        git("apply", "--check", str(root / entry["patch"]))
        git("apply", str(root / entry["patch"]))
print("Pinned source trees and contribution patches are ready.")
