#!/usr/bin/env python3
"""
Download ONNX models for ELID embedding support.
Usage: python scripts/download_models.py [--force] [--text-only] [--image-only]
"""

import sys
from pathlib import Path

try:
    from huggingface_hub import hf_hub_download
except ImportError:
    print("Installing huggingface_hub...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "huggingface_hub"])
    from huggingface_hub import hf_hub_download

MODELS_DIR = Path(__file__).parent.parent / "models"

MODELS = {
    "text": {
        "repo": "minishlab/potion-base-8M",
        "filename": "model.onnx",
        "local_name": "potion-base-8m.onnx",
        "description": "Model2Vec text embeddings (8MB, 256-dim)",
    },
    "image": {
        "repo": "timm/mobilenetv3_small_100.lamb_in1k",
        "filename": "model.onnx",
        "local_name": "mobilenetv3-small.onnx",
        "description": "MobileNetV3 image features (5MB, 1024-dim)",
    },
}


def download_model(name: str, config: dict, force: bool = False) -> Path:
    """Download a model from HuggingFace Hub."""
    dest = MODELS_DIR / config["local_name"]

    if dest.exists() and not force:
        size_mb = dest.stat().st_size / (1024 * 1024)
        print(f"✓ {name}: {config['local_name']} ({size_mb:.1f}MB, already exists)")
        return dest

    print(f"⬇ {name}: Downloading {config['description']}...")

    downloaded = hf_hub_download(
        repo_id=config["repo"],
        filename=config["filename"],
        local_dir=MODELS_DIR,
        local_dir_use_symlinks=False,
    )

    # Rename if needed
    downloaded_path = Path(downloaded)
    if downloaded_path.name != config["local_name"]:
        downloaded_path.rename(dest)

    size_mb = dest.stat().st_size / (1024 * 1024)
    print(f"✓ {name}: {config['local_name']} ({size_mb:.1f}MB)")
    return dest


def main():
    args = set(sys.argv[1:])
    force = "--force" in args
    text_only = "--text-only" in args
    image_only = "--image-only" in args

    MODELS_DIR.mkdir(exist_ok=True)

    print("ELID Model Downloader")
    print("=" * 50)

    # Determine which models to download
    if text_only:
        models_to_download = {"text": MODELS["text"]}
    elif image_only:
        models_to_download = {"image": MODELS["image"]}
    else:
        models_to_download = MODELS

    success = True
    for name, config in models_to_download.items():
        try:
            download_model(name, config, force)
        except Exception as e:
            print(f"✗ {name}: Failed - {e}")
            success = False

    print("=" * 50)
    if success:
        print(f"Models ready in {MODELS_DIR}/")
        print("\nNext steps:")
        print("  cargo check --features 'embeddings models'")
        print("  wasm-pack build --target web -- --features 'wasm models-text models-image'")
        return 0
    else:
        print("Some models failed to download.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
