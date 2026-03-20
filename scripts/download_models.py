#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = ["huggingface_hub", "requests", "onnx"]
# ///
"""
Download models for ELID embedding support.
Usage: uv run --script scripts/download_models.py [--force] [--text-only] [--image-only]
"""

import sys
from pathlib import Path

import requests
from huggingface_hub import hf_hub_download

MODELS_DIR = Path(__file__).parent.parent / "models"

# Text model: Model2Vec potion-base-8M (safetensors + tokenizer)
TEXT_FILES = [
    {
        "repo": "minishlab/potion-base-8M",
        "filename": "model.safetensors",
        "local_name": "potion-base-8m.safetensors",
        "description": "Model2Vec embedding matrix (28MB, 256-dim)",
    },
    {
        "repo": "minishlab/potion-base-8M",
        "filename": "tokenizer.json",
        "local_name": "potion-base-8m-tokenizer.json",
        "description": "WordPiece tokenizer config",
    },
    {
        "repo": "minishlab/potion-base-8M",
        "filename": "config.json",
        "local_name": "potion-base-8m-config.json",
        "description": "Model2Vec config (hidden_dim, normalize, etc.)",
    },
]

# Image model: MobileNetV3-Small from Qualcomm AI Hub (tract-compatible ONNX export)
IMAGE_MODEL_URL = "https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-models/models/mobilenet_v3_small/releases/v0.48.0/mobilenet_v3_small-onnx-float.zip"
IMAGE_MODEL_ONNX = "mobilenetv3-small.onnx"


def download_hf_file(config: dict, force: bool = False) -> Path:
    """Download a single file from HuggingFace Hub."""
    dest = MODELS_DIR / config["local_name"]

    if dest.exists() and not force:
        size_mb = dest.stat().st_size / (1024 * 1024)
        print(f"  [ok] {config['local_name']} ({size_mb:.1f}MB, already exists)")
        return dest

    print(f"  [dl] {config['description']}...")

    downloaded = hf_hub_download(
        repo_id=config["repo"],
        filename=config["filename"],
        local_dir=MODELS_DIR,
    )

    # Rename if needed
    downloaded_path = Path(downloaded)
    if downloaded_path.name != config["local_name"]:
        dest.parent.mkdir(parents=True, exist_ok=True)
        downloaded_path.rename(dest)

    size_mb = dest.stat().st_size / (1024 * 1024)
    print(f"  [ok] {config['local_name']} ({size_mb:.1f}MB)")
    return dest


def download_image_model(force: bool = False) -> Path:
    """Download MobileNetV3-Small ONNX from Qualcomm AI Hub and inline weights."""
    import io
    import tempfile
    import zipfile

    import onnx

    onnx_dest = MODELS_DIR / IMAGE_MODEL_ONNX

    if onnx_dest.exists() and not force:
        size_mb = onnx_dest.stat().st_size / (1024 * 1024)
        print(f"  [ok] {IMAGE_MODEL_ONNX} ({size_mb:.1f}MB, already exists)")
        return onnx_dest

    print(f"  [dl] MobileNetV3-Small ONNX (10MB, 1000-dim)...")

    resp = requests.get(IMAGE_MODEL_URL, allow_redirects=True, timeout=120)
    resp.raise_for_status()

    # Extract to temp dir, then inline weights into single ONNX
    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)
        with zipfile.ZipFile(io.BytesIO(resp.content)) as zf:
            zf.extractall(tmpdir)

        # Find the ONNX file in extracted contents
        onnx_files = list(tmpdir.rglob("*.onnx"))
        if not onnx_files:
            raise FileNotFoundError("No .onnx file found in downloaded archive")

        # Load with external data, then save as self-contained
        onnx_path = onnx_files[0]
        model = onnx.load(str(onnx_path), load_external_data=True)
        onnx.save(model, str(onnx_dest))

    size_mb = onnx_dest.stat().st_size / (1024 * 1024)
    print(f"  [ok] {IMAGE_MODEL_ONNX} ({size_mb:.1f}MB, weights inlined)")
    return onnx_dest


def main():
    args = set(sys.argv[1:])
    force = "--force" in args
    text_only = "--text-only" in args
    image_only = "--image-only" in args

    MODELS_DIR.mkdir(exist_ok=True)

    print("ELID Model Downloader")
    print("=" * 50)

    success = True

    if not image_only:
        print("\nText model (Model2Vec potion-base-8M):")
        for config in TEXT_FILES:
            try:
                download_hf_file(config, force)
            except Exception as e:
                print(f"  [!!] {config['local_name']}: Failed - {e}")
                success = False

    if not text_only:
        print("\nImage model (MobileNetV3-Small, Qualcomm AI Hub):")
        try:
            download_image_model(force)
        except Exception as e:
            print(f"  [!!] {IMAGE_MODEL_ONNX}: Failed - {e}")
            success = False

    print("\n" + "=" * 50)
    if success:
        print(f"Models ready in {MODELS_DIR}/")
        print("\nUsage:")
        if not image_only:
            print("  cargo check --features models-text")
        if not text_only:
            print("  cargo check --features models-image")
        return 0
    else:
        print("Some files failed to download.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
