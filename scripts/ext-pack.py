"""Zip the GNOME extension for extensions.gnome.org (`make ext-pack`).

Ships what `install-gnome-extension` installs: everything under `extension/` except the dev-only
entries (`DEV_ONLY` in `src/install/gnome.rs`). Fixed timestamps and sorted entries, so the same
tree always gives the same zip.
"""

import sys
import zipfile
from pathlib import Path

DEV_ONLY = {"tests", "node_modules", "package.json", "package-lock.json", "jsconfig.json", "eslint.config.js"}
EPOCH = (1980, 1, 1, 0, 0, 0)


def main(src: Path, out: Path) -> None:
    files = sorted(
        p for p in src.rglob("*")
        if p.is_file() and p.relative_to(src).parts[0] not in DEV_ONLY
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        for p in files:
            info = zipfile.ZipInfo(str(p.relative_to(src)), EPOCH)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            z.writestr(info, p.read_bytes())
    print(out)


if __name__ == "__main__":
    main(Path(sys.argv[1]), Path(sys.argv[2]))
