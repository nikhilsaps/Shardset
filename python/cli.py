"""Shardset CLI: reproduce an image with geometric shards.

Python drives; the compiled Rust engine (shardset_engine) does the multicore work.

Example:
    python cli.py -i input.jpg -o out.png -o out.svg -n 200 -m 1
"""

import argparse
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _bootstrap import ensure
ensure()  # auto-install pillow + the Rust engine if missing

from PIL import Image
import shardset_engine as pe

MODE_NAMES = {
    0: "combo", 1: "triangle", 2: "rectangle", 3: "ellipse", 4: "circle",
    5: "rotated-rectangle", 6: "bezier", 7: "rotated-ellipse", 8: "polygon",
}


def load_target(path: str, input_size: int):
    img = Image.open(path).convert("RGBA")
    if input_size > 0:
        img.thumbnail((input_size, input_size), Image.LANCZOS)
    w, h = img.size
    return w, h, img.tobytes()


def save_png(model, path: str):
    img = Image.frombytes("RGBA", (model.width, model.height), model.rgba_bytes())
    img.save(path)


def save_svg(model, path: str):
    with open(path, "w", encoding="utf-8") as fp:
        fp.write(model.svg())


def main(argv=None):
    p = argparse.ArgumentParser(description="Reproduce an image with geometric shards (Shardset, Rust engine).")
    p.add_argument("-i", "--input", required=True, help="input image path")
    p.add_argument("-o", "--output", action="append", required=True,
                   help="output path (.png or .svg); repeatable")
    p.add_argument("-n", "--number", type=int, required=True, help="number of shapes")
    p.add_argument("-m", "--mode", type=int, default=1,
                   help="0=combo 1=triangle 2=rect 3=ellipse 4=circle "
                        "5=rotatedrect 6=bezier 7=rotatedellipse 8=polygon")
    p.add_argument("-a", "--alpha", type=int, default=128, help="alpha (0 = auto)")
    p.add_argument("-r", "--input-size", type=int, default=256, help="resize input to this")
    p.add_argument("-rep", "--repeat", type=int, default=0, help="extra refined shapes per step")
    p.add_argument("-j", "--workers", type=int, default=0, help="worker threads (0 = all cores)")
    p.add_argument("-bg", "--background", default=None, help="background hex color")
    p.add_argument("-v", "--verbose", action="store_true")
    args = p.parse_args(argv)

    w, h, rgba = load_target(args.input, args.input_size)
    workers = args.workers if args.workers and args.workers > 0 else None
    model = pe.Model(w, h, rgba, args.background, workers)

    print(f"input {args.input} ({w}x{h}), bg {model.background_hex}, "
          f"{model.workers} workers, mode {MODE_NAMES.get(args.mode, args.mode)}")
    print(f"{0:4d}: score={model.score:.6f}")

    start = time.time()
    for i in range(1, args.number + 1):
        t = time.time()
        n = model.step(args.mode, args.alpha, args.repeat)
        if args.verbose or i == args.number or i % 10 == 0:
            dt = time.time() - t
            nps = n / dt if dt > 0 else 0
            print(f"{i:4d}: score={model.score:.6f}  n={n}  {nps/1000:.1f}k/s  "
                  f"elapsed={time.time()-start:.1f}s")

    for out in args.output:
        low = out.lower()
        if low.endswith(".svg"):
            save_svg(model, out)
        else:
            save_png(model, out)
        print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
