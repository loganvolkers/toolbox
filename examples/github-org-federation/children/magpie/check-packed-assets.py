"""Verify Magpie's release packer with a small embedded GUI asset tree."""

import subprocess
import tempfile
import zipfile
from pathlib import Path

assets = {"index.html": b"<main>Magpie</main>", "css/app.css": b"main { display: block; }"}
excluded = (".local", "_build/ignored.txt", "css/.cache")
with tempfile.TemporaryDirectory(prefix="magpie-asset-check-") as directory:
    root = Path(directory)
    source = root / "assets"
    for name, content in assets.items():
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
    for name in excluded:
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"excluded")
    pack = root / "assets.zip"
    subprocess.run(
        ["go", "run", "./build/assetpack/main.go", str(source), str(pack)], check=True
    )
    expected = {f"assets/{name}": content for name, content in assets.items()}
    with zipfile.ZipFile(pack) as archive:
        if sorted(archive.namelist()) != sorted(expected):
            raise SystemExit(
                f"asset archive entries differ from the embedded tree: {archive.namelist()}"
            )
        for name, data in expected.items():
            if archive.read(name) != data:
                raise SystemExit(f"asset archive changed the bytes of {name}")
