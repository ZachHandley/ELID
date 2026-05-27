#!/usr/bin/env python3
"""Generate GitHub Actions workflows from the .forgejo/workflows sources.

The Forgejo workflows lean on org-internal composite actions and Forgejo-only
jobs that have no GitHub equivalent:

  * setup-rust@main      -> dtolnay/rust-toolchain@stable, dropping the sccache
                            S3 inputs and install-gcc (gcc is preinstalled on
                            GitHub's ubuntu runners).
  * setup-node@main      -> pnpm/action-setup + actions/setup-node. The Forgejo
                            action bundles pnpm; on GitHub pnpm must be installed
                            explicitly before setup-node can cache it.
  * setup-python@main    -> actions/setup-python + astral-sh/setup-uv. The
                            Forgejo action bundles uv; GitHub needs it added.
  * gitea-*-artifact     -> actions/{upload,download}-artifact (matched v4 pair).
  * actions/checkout@v4  -> @v6 (avoids the Node20 deprecation forcing).
  * trigger-docs / trigger-release jobs (ci.yml) hit the Forgejo REST API to
    dispatch sibling workflows; they are stripped from the mirror.

docs.yml and release.yml are intentionally NOT mirrored: they build artifacts on
Forgejo and push them *to* GitHub, so running them *on* GitHub is circular and
depends on Forgejo-only secrets.

Run from the repo root. Writes a fresh .github/workflows/ tree.
"""

from __future__ import annotations

import re
import shutil
from pathlib import Path

SRC = Path(".forgejo/workflows")
DST = Path(".github/workflows")
FORGE = "https://forge.blackleafdigital.com/Public/actions"

# Only these run meaningfully on the GitHub mirror.
WORKFLOWS = ["ci.yml", "benchmark.yml", "security.yml"]

# Job keys (indent-2) that only make sense on Forgejo.
DROP_JOBS = {"trigger-docs", "trigger-release"}

# setup-rust `with:` keys that don't exist on dtolnay/rust-toolchain.
DROP_RUST_WITH = ("sccache", "install-gcc")


def _setup_node(m: re.Match) -> str:
    i = m.group("i")
    ver = m.group("ver")
    return (
        f"{i}- name: Install pnpm\n"
        f"{i}  uses: pnpm/action-setup@v6\n"
        f"{i}- name: Setup Node.js\n"
        f"{i}  uses: actions/setup-node@v6\n"
        f"{i}  with:\n"
        f"{i}    node-version: {ver}\n"
        f"{i}    cache: pnpm\n"
    )


def _setup_python(m: re.Match) -> str:
    i = m.group("i")
    ver = m.group("ver")
    return (
        f"{i}- name: Setup Python\n"
        f"{i}  uses: actions/setup-python@v6\n"
        f"{i}  with:\n"
        f"{i}    python-version: {ver}\n"
        f"{i}- name: Install uv\n"
        f"{i}  uses: astral-sh/setup-uv@v8\n"
    )


def transform(text: str, filename: str) -> str:
    # Simple string swaps.
    text = text.replace(
        "https://github.com/christopherhx/gitea-upload-artifact@v4",
        "actions/upload-artifact@v4",
    )
    text = text.replace(
        "https://github.com/christopherhx/gitea-download-artifact@v4",
        "actions/download-artifact@v4",
    )
    text = text.replace("actions/checkout@v4", "actions/checkout@v6")

    # Expand setup-node / setup-python steps (node-version is '22', python '3.12',
    # but capture the value so we don't silently drift if the source changes).
    fnode = re.escape(f"{FORGE}/setup-node@main")
    text = re.sub(
        rf"^(?P<i> *)- name: Setup Node\.js\n"
        rf"(?P=i)  uses: {fnode}\n"
        rf"(?P=i)  with:\n"
        rf"(?P=i)    node-version: (?P<ver>.+)\n",
        _setup_node,
        text,
        flags=re.MULTILINE,
    )
    fpy = re.escape(f"{FORGE}/setup-python@main")
    text = re.sub(
        rf"^(?P<i> *)- name: Setup Python\n"
        rf"(?P=i)  uses: {fpy}\n"
        rf"(?P=i)  with:\n"
        rf"(?P=i)    python-version: (?P<ver>.+)\n",
        _setup_python,
        text,
        flags=re.MULTILINE,
    )

    # setup-rust uses-line swap.
    text = text.replace(
        f"{FORGE}/setup-rust@main", "dtolnay/rust-toolchain@stable"
    )

    # Line pass: clean dtolnay `with:` blocks (drop sccache/install-gcc; drop an
    # empty `with:`), and drop Forgejo-only jobs.
    lines = text.split("\n")
    out: list[str] = []
    i = 0
    n = len(lines)
    while i < n:
        line = lines[i]
        stripped = line.strip()

        # Drop a whole job block (indent-2 key) we don't want on the mirror.
        m = re.match(r"^  ([A-Za-z0-9_-]+):\s*$", line)
        if m and m.group(1) in DROP_JOBS:
            i += 1
            while i < n and not re.match(r"^  \S", lines[i]):
                i += 1
            # also swallow a trailing blank line for tidiness
            while out and out[-1].strip() == "":
                out.pop()
            out.append("")
            continue

        out.append(line)

        # When we emit a dtolnay uses line, sanitize the following with: block.
        if stripped == "uses: dtolnay/rust-toolchain@stable":
            uses_indent = len(line) - len(line.lstrip())
            # Peek: is the next line a `with:` at uses_indent?
            if i + 1 < n:
                nxt = lines[i + 1]
                if nxt.strip() == "with:" and (len(nxt) - len(nxt.lstrip())) == uses_indent:
                    with_indent = uses_indent
                    j = i + 2
                    kept: list[str] = []
                    while j < n:
                        child = lines[j]
                        if child.strip() == "":
                            # blank line ends nothing definitively; stop the block
                            break
                        cind = len(child) - len(child.lstrip())
                        if cind <= with_indent:
                            break
                        key = child.strip().split(":", 1)[0]
                        if not key.startswith(DROP_RUST_WITH):
                            kept.append(child)
                        j += 1
                    if kept:
                        out.append(nxt)  # the `with:` line
                        out.extend(kept)
                    # else: drop the empty `with:` entirely
                    i = j - 1  # continue after the consumed block
        i += 1

    return "\n".join(out)


def main() -> None:
    if DST.exists():
        shutil.rmtree(DST)
    DST.mkdir(parents=True)
    for name in WORKFLOWS:
        src = SRC / name
        if not src.exists():
            continue
        (DST / name).write_text(transform(src.read_text(), name))
        print(f"generated .github/workflows/{name}")


if __name__ == "__main__":
    main()
