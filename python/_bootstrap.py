"""Dependency bootstrap for Shardset.

Both cli.py and gui.py call ensure() before importing their dependencies. Anything
missing is installed into the *current* Python interpreter automatically, so you
never hit a "No module named ..." error:

  - pillow / flet  -> pip install
  - shardset_engine (the Rust core) -> installed from the prebuilt wheel in
    rust/target/wheels/, or built with maturin if no wheel is present
    (building needs the Rust toolchain: https://rustup.rs).
"""
import glob
import importlib
import os
import subprocess
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
_RUST = os.path.join(_HERE, os.pardir, "rust")


def _pip(*args):
    subprocess.check_call([sys.executable, "-m", "pip", "install", *args])


def _have(mod):
    try:
        importlib.import_module(mod)
        return True
    except ImportError:
        return False


def _ensure_pip_pkg(module, package):
    if not _have(module):
        print(f"[shardset] installing {package} ...", flush=True)
        _pip(package)
        importlib.invalidate_caches()


def _engine_wheel():
    found = sorted(glob.glob(os.path.join(_RUST, "target", "wheels", "shardset_engine-*.whl")))
    return found[-1] if found else None


def _ensure_engine():
    if _have("shardset_engine"):
        return
    wheel = _engine_wheel()
    if wheel is None:
        _ensure_pip_pkg("maturin", "maturin")
        print("[shardset] no prebuilt wheel found - building Rust engine with maturin ...", flush=True)
        subprocess.check_call([sys.executable, "-m", "maturin", "build", "--release"], cwd=_RUST)
        wheel = _engine_wheel()
    if wheel is None:
        raise RuntimeError(
            "shardset_engine is missing and could not be built. "
            "Install the Rust toolchain (https://rustup.rs), then run: "
            "cd rust && maturin develop --release"
        )
    print(f"[shardset] installing engine: {os.path.basename(wheel)}", flush=True)
    _pip("--force-reinstall", "--no-deps", wheel)
    importlib.invalidate_caches()


def ensure(flet=False):
    """Ensure runtime deps are importable. Set flet=True for the GUI."""
    _ensure_pip_pkg("PIL", "pillow")
    if flet:
        _ensure_pip_pkg("flet", "flet")
    _ensure_engine()
