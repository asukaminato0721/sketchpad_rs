#!/usr/bin/env python3
"""Package the native binaries produced by the rolling release workflow."""

import argparse
import json
from pathlib import Path
import plistlib
import shutil
import tarfile
import tempfile
import tomllib
import zipfile


TARGETS = {
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
}
ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=sorted(TARGETS))
    parser.add_argument("--commit", required=True)
    parser.add_argument("--binary-dir", type=Path)
    parser.add_argument("--output-dir", type=Path, default=ROOT / "dist")
    args = parser.parse_args()
    binary_dir = args.binary_dir or ROOT / "target" / args.target / "release"
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    windows = args.target.endswith("windows-msvc")
    macos = args.target.endswith("apple-darwin")
    suffix = ".exe" if windows else ""
    # Check both inputs before creating the archive.
    for name in ("sketchpad_rs", "gsp_probe"):
        if not (binary_dir / (name + suffix)).is_file():
            parser.error(f"Missing binary: {binary_dir / (name + suffix)}")

    args.output_dir.mkdir(parents=True, exist_ok=True)
    name = f"sketchpad-rs-{args.target}"
    with tempfile.TemporaryDirectory() as temporary:
        package = Path(temporary) / name
        package.mkdir()
        editor_dir = package
        if macos:
            contents = package / "Sketchpad RS.app" / "Contents"
            editor_dir = contents / "MacOS"
            editor_dir.mkdir(parents=True)
            with (contents / "Info.plist").open("wb") as file:
                plistlib.dump({
                    "CFBundleName": "Sketchpad RS",
                    "CFBundleDisplayName": "Sketchpad RS",
                    "CFBundleIdentifier": "io.github.asukaminato0721.sketchpad-rs",
                    "CFBundleExecutable": "sketchpad_rs",
                    "CFBundlePackageType": "APPL",
                    "CFBundleShortVersionString": version,
                    "CFBundleVersion": version,
                    "NSHighResolutionCapable": True,
                }, file)
        for binary, destination in (("sketchpad_rs", editor_dir), ("gsp_probe", package)):
            output = destination / (binary + suffix)
            shutil.copy2(binary_dir / output.name, output)
            output.chmod(0o755)
        shutil.copy2(ROOT / "README.md", package / "README.md")
        notices = package / "licenses" / "NotoSansCJK"
        notices.mkdir(parents=True)
        for notice in ("README.md", "OFL.txt"):
            shutil.copy2(ROOT / "assets" / "fonts" / notice, notices / notice)
        if macos:
            shutil.copytree(package / "licenses", contents / "Resources" / "licenses")
        (package / "BUILD-INFO.json").write_text(
            json.dumps({"version": version, "commit": args.commit, "target": args.target}, indent=2) + "\n",
            encoding="utf-8",
        )
        if windows:
            archive = args.output_dir / f"{name}.zip"
            with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as file:
                for path in sorted(package.rglob("*")):
                    if path.is_file():
                        file.write(path, path.relative_to(package.parent))
        else:
            archive = args.output_dir / f"{name}.tar.gz"
            with tarfile.open(archive, "w:gz") as file:
                file.add(package, arcname=name)
        print(archive)


if __name__ == "__main__":
    main()
