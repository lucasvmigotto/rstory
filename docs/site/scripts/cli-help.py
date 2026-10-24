"""Generate the CLI reference data from the real binary's --help output.

Usage: python3 -I cli-help.py <rstory binary> <output json>

Run it through `make docs-reference` (tools image, offline). Never edit the output by hand:
CI regenerates it and fails when it differs.
"""
import json
import os
import re
import shutil
import subprocess
import sys

binary, out_path = sys.argv[1], sys.argv[2]
root = "/tmp/rstory-docs-reference"
shutil.rmtree(root, ignore_errors=True)
for d in ("home", "state", "config", "data", "run"):
    os.makedirs(os.path.join(root, d), mode=0o700)

ENV = {
    "PATH": "/usr/bin:/bin",
    "HOME": os.path.join(root, "home"),
    "XDG_STATE_HOME": os.path.join(root, "state"),
    "XDG_CONFIG_HOME": os.path.join(root, "config"),
    "XDG_DATA_HOME": os.path.join(root, "data"),
    "XDG_RUNTIME_DIR": os.path.join(root, "run"),
    "NO_COLOR": "1",
    "COLUMNS": "100",
    "LANG": "C.UTF-8",
    "TZ": "UTC",
}


def run(*args):
    p = subprocess.run(
        [binary, *args], env=ENV, stdin=subprocess.DEVNULL, capture_output=True, timeout=30
    )
    return p.returncode, p.stdout.decode("utf-8"), p.stderr.decode("utf-8")


def text(code, out, err):
    """What a reader sees: stdout, or stderr when stdout is empty."""
    return out.rstrip("\n") if out.strip() else err.rstrip("\n")


def scrub(s):
    return s.replace(root, "<tmp>")


code, out, err = run("--help")
assert code == 0, "rstory --help failed"
names = []
in_block = False
for line in out.splitlines():
    if line.startswith("Commands:"):
        in_block = True
        continue
    if in_block:
        m = re.match(r"^  ([a-z][a-z-]*)\b", line)
        if not m:
            break
        if m.group(1) != "help":
            names.append(m.group(1))
assert names, "no commands found in `rstory --help`"

commands = [{"name": "rstory", "args": ["--help"], "exit": code, "help": text(code, out, err)}]
for name in names:
    c, o, e = run(name, "--help")
    commands.append({"name": name, "args": [name, "--help"], "exit": c, "help": text(c, o, e)})

_, version, _ = run("--version")
c_cfg, cfg, _ = run("config")
c_doc, doc, doc_err = run("doctor", "--json")
try:
    checks = json.loads(doc)["checks"]
    doctor = [
        {"id": c["id"], "status": c["status"], "message": scrub(c["message"]), "hint": scrub(c.get("hint") or "")}
        for c in checks
    ]
except Exception:
    doctor = []
shells = {}
for sh in ("bash", "zsh", "fish"):
    c, o, _ = run("completions", sh)
    shells[sh] = c == 0 and len(o) > 0

data = {
    "_generated": "make docs-reference - do not hand-edit",
    "rstory": version.strip().removeprefix("rstory ").strip(),
    "commands": commands,
    "config_defaults": scrub(cfg.rstrip("\n")),
    "doctor": {"exit": c_doc, "checks": doctor},
    "completions": shells,
}
with open(out_path, "w") as f:
    json.dump(data, f, indent=2, ensure_ascii=False)
    f.write("\n")
print(f"wrote {out_path}: {len(commands)} commands", file=sys.stderr)
