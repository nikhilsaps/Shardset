"""Shardset GUI: reproduce an image with geometric shards.

Python + Flet drive the UI; the compiled Rust engine (shardset_engine) runs the
optimization across all CPU cores. Launch with:  python gui.py
"""

import base64
import io
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _bootstrap import ensure
ensure(flet=True)  # auto-install flet + pillow + the Rust engine if missing

import asyncio

import flet as ft
from PIL import Image
import shardset_engine as pe

MODES = [
    ("1", "Triangle"), ("2", "Rectangle"), ("3", "Ellipse"), ("4", "Circle"),
    ("5", "Rotated Rectangle"), ("6", "Bezier"), ("7", "Rotated Ellipse"),
    ("8", "Polygon"), ("0", "Combo (all)"),
]

ACCENT = "#4f46e5"       # indigo
ACCENT_SOFT = "#eef2ff"  # indigo-50
PANEL_BG = "#f1f5f9"     # slate-100
TRACK_BG = "#e2e8f0"     # slate-200
MUTED = "#64748b"        # slate-500

# Target duration of the build-up animation so a run you otherwise couldn't see
# (the engine finishes in a blink) plays out as a visible "painting" of shards.
ANIM_SECONDS = 4.0
TARGET_FRAMES = 110


def pil_to_src(img: Image.Image) -> str:
    """Encode a PIL image as a data URI Flet's Image.src accepts."""
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    b64 = base64.b64encode(buf.getvalue()).decode("ascii")
    return f"data:image/png;base64,{b64}"


def main(page: ft.Page):
    page.title = "Shardset - image to shards (Rust engine)"
    page.theme_mode = ft.ThemeMode.LIGHT
    page.theme = ft.Theme(color_scheme_seed=ACCENT)
    page.bgcolor = "#ffffff"
    page.padding = 16
    page.window.width = 1180
    page.window.height = 840

    state = {"target": None, "rgba": None, "size": (0, 0),
             "model": None, "running": False, "stop": False}

    # ---- Controls -------------------------------------------------------
    path_field = ft.TextField(label="Image path", expand=True, dense=True)
    picker = ft.FilePicker()
    page.services.append(picker)

    mode_dd = ft.Dropdown(
        label="Shape",
        value="1",
        options=[ft.DropdownOption(key=k, text=t) for k, t in MODES],
        width=220,
    )
    count_slider = ft.Slider(min=1, max=1000, divisions=999, value=100,
                             label="{value}", expand=True, active_color=ACCENT)
    count_label = ft.Text("Shards: 100", width=120)
    alpha_slider = ft.Slider(min=0, max=255, divisions=255, value=128,
                             label="{value}", expand=True, active_color=ACCENT)
    alpha_label = ft.Text("Alpha: 128 ", width=120)
    size_field = ft.TextField(label="Input size", value="256", width=110, dense=True)
    repeat_field = ft.TextField(label="Repeat", value="0", width=90, dense=True)
    workers_field = ft.TextField(label="Workers (0=all)", value="0", width=130, dense=True)
    animate_sw = ft.Switch(label="Watch it build", value=True, active_color=ACCENT)

    status = ft.Text("Load an image to begin.", selectable=True, color=MUTED, size=12)

    # ---- Progress panel -------------------------------------------------
    count_big = ft.Text("0", size=30, weight=ft.FontWeight.BOLD, color=ACCENT)
    count_total = ft.Text("/ 0 shards", size=14, color=MUTED)
    progress = ft.ProgressBar(value=0, bar_height=12, color=ACCENT,
                              bgcolor=TRACK_BG, border_radius=6)
    pct_text = ft.Text("0%", size=12, color=MUTED)
    score_metric = ft.Text("-", size=15, weight=ft.FontWeight.BOLD)
    time_metric = ft.Text("-", size=15, weight=ft.FontWeight.BOLD)
    speed_metric = ft.Text("-", size=15, weight=ft.FontWeight.BOLD)

    def metric(label, value_ctrl):
        return ft.Column(
            [ft.Text(label, size=10, color=MUTED), value_ctrl],
            spacing=1, horizontal_alignment=ft.CrossAxisAlignment.CENTER,
            expand=True,
        )

    progress_card = ft.Container(
        content=ft.Column(
            [
                ft.Row([count_big, count_total, ft.Container(expand=True), pct_text],
                       vertical_alignment=ft.CrossAxisAlignment.END),
                progress,
                ft.Row([metric("SCORE (lower=better)", score_metric),
                        metric("ELAPSED", time_metric),
                        metric("SHARDS/SEC", speed_metric)]),
            ],
            spacing=10,
        ),
        bgcolor=ACCENT_SOFT, border_radius=12, padding=16,
    )

    cores = os.cpu_count() or 1
    cores_text = ft.Text(f"CPU cores available: {cores}", size=12, color=MUTED)

    # ---- Image panels (content swaps between placeholder and image) -----
    original_img = ft.Image(src="", fit=ft.BoxFit.CONTAIN, expand=True)
    result_img = ft.Image(src="", fit=ft.BoxFit.CONTAIN, expand=True)

    def placeholder(text, icon=ft.Icons.IMAGE_OUTLINED):
        return ft.Column(
            [ft.Icon(icon, size=56, color="#cbd5e1"),
             ft.Text(text, size=12, color=MUTED)],
            alignment=ft.MainAxisAlignment.CENTER,
            horizontal_alignment=ft.CrossAxisAlignment.CENTER,
            expand=True,
        )

    orig_box = ft.Container(content=placeholder("No image loaded"),
                            expand=True, padding=6, bgcolor=PANEL_BG,
                            border_radius=8)
    result_box = ft.Container(content=placeholder("Run to reconstruct",
                                                   ft.Icons.AUTO_AWESOME_MOSAIC),
                              expand=True, padding=6, bgcolor=PANEL_BG,
                              border_radius=8)

    run_btn = ft.FilledButton("Run", icon=ft.Icons.PLAY_ARROW, disabled=True)
    stop_btn = ft.OutlinedButton("Stop", icon=ft.Icons.STOP, disabled=True)
    save_png_btn = ft.OutlinedButton("Save PNG", icon=ft.Icons.IMAGE, disabled=True)
    save_svg_btn = ft.OutlinedButton("Save SVG", icon=ft.Icons.CODE, disabled=True)

    # ---- Image loading --------------------------------------------------
    def load_image(path: str):
        if not path or not os.path.isfile(path):
            status.value = f"File not found: {path}"
            page.update()
            return
        try:
            size = int(size_field.value or "256")
        except ValueError:
            size = 256
        img = Image.open(path).convert("RGBA")
        if size > 0:
            img.thumbnail((size, size), Image.LANCZOS)
        state["target"] = img
        state["rgba"] = img.tobytes()
        state["size"] = img.size
        original_img.src = pil_to_src(img)
        orig_box.content = original_img
        result_img.src = ""
        result_box.content = placeholder("Run to reconstruct", ft.Icons.AUTO_AWESOME_MOSAIC)
        run_btn.disabled = False
        save_png_btn.disabled = True
        save_svg_btn.disabled = True
        count_big.value = "0"
        count_total.value = f"/ {int(count_slider.value)} shards"
        progress.value = 0
        pct_text.value = "0%"
        score_metric.value = time_metric.value = speed_metric.value = "-"
        status.value = f"Loaded {os.path.basename(path)}  ({img.size[0]}x{img.size[1]})"
        page.update()

    def on_load_click(e):
        load_image(path_field.value.strip().strip('"'))

    async def on_browse(e):
        files = await picker.pick_files(
            dialog_title="Choose an image",
            allowed_extensions=["png", "jpg", "jpeg", "bmp", "webp", "gif"],
            allow_multiple=False,
        )
        if files:
            path_field.value = files[0].path
            load_image(files[0].path)

    # ---- Optimization worker (async so each frame renders) --------------
    async def worker():
        w, h = state["size"]
        try:
            workers = int(workers_field.value or "0")
        except ValueError:
            workers = 0
        try:
            repeat = int(repeat_field.value or "0")
        except ValueError:
            repeat = 0
        mode = int(mode_dd.value)
        alpha = int(alpha_slider.value)
        total = int(count_slider.value)
        animate = animate_sw.value

        model = pe.Model(w, h, state["rgba"], None, workers or None)
        state["model"] = model
        result_box.content = result_img

        frames = min(total, TARGET_FRAMES)
        update_every = max(1, total // frames)
        frame_delay = (ANIM_SECONDS / frames) if animate else 0.0

        start = time.time()
        for i in range(1, total + 1):
            if state["stop"]:
                break
            model.step(mode, alpha, repeat)
            if i % update_every == 0 or i == total:
                img = Image.frombytes("RGBA", (w, h), model.rgba_bytes())
                result_img.src = pil_to_src(img)
                elapsed = time.time() - start
                progress.value = i / total
                count_big.value = str(i)
                count_total.value = f"/ {total} shards"
                pct_text.value = f"{100 * i // total}%"
                score_metric.value = f"{model.score:.5f}"
                time_metric.value = f"{elapsed:.1f}s"
                speed_metric.value = f"{(i / elapsed):.0f}" if elapsed > 0 else "-"
                status.value = (f"{model.workers} workers  |  bg {model.background_hex}  "
                                f"|  mode {mode}")
                page.update()
                # Yield to the UI loop so this frame actually paints.
                await asyncio.sleep(frame_delay if frame_delay else 0.001)

        stopped = state["stop"]
        state["running"] = False
        state["stop"] = False
        run_btn.disabled = False
        stop_btn.disabled = True
        save_png_btn.disabled = False
        save_svg_btn.disabled = False
        if stopped:
            status.value = f"Stopped at {model.shape_count} shards."
        else:
            status.value = (f"Done: {model.shape_count} shards  |  final score "
                            f"{model.score:.5f}  |  {time.time() - start:.1f}s")
        page.update()

    def on_run(e):
        if state["rgba"] is None or state["running"]:
            return
        state["running"] = True
        state["stop"] = False
        run_btn.disabled = True
        stop_btn.disabled = False
        save_png_btn.disabled = True
        save_svg_btn.disabled = True
        progress.value = 0
        count_big.value = "0"
        pct_text.value = "0%"
        page.update()
        page.run_task(worker)

    def on_stop(e):
        state["stop"] = True
        stop_btn.disabled = True
        page.update()

    # ---- Saving ---------------------------------------------------------
    def default_out(ext: str) -> str:
        base = path_field.value.strip().strip('"') or "output"
        root, _ = os.path.splitext(base)
        return f"{root}.shardset.{ext}"

    def on_save_png(e):
        model = state["model"]
        if not model:
            return
        out = default_out("png")
        w, h = state["size"]
        Image.frombytes("RGBA", (w, h), model.rgba_bytes()).save(out)
        status.value = f"Saved {out}"
        page.update()

    def on_save_svg(e):
        model = state["model"]
        if not model:
            return
        out = default_out("svg")
        with open(out, "w", encoding="utf-8") as fp:
            fp.write(model.svg())
        status.value = f"Saved {out}"
        page.update()

    # ---- Wiring ---------------------------------------------------------
    def on_count_change(e):
        n = int(count_slider.value)
        count_label.value = f"Shards: {n}"
        count_total.value = f"/ {n} shards"
        page.update()

    def on_alpha_change(e):
        v = int(alpha_slider.value)
        alpha_label.value = f"Alpha: {v}{' (auto)' if v == 0 else ''}"
        page.update()

    count_slider.on_change = on_count_change
    alpha_slider.on_change = on_alpha_change
    run_btn.on_click = on_run
    stop_btn.on_click = on_stop
    save_png_btn.on_click = on_save_png
    save_svg_btn.on_click = on_save_svg

    browse_btn = ft.OutlinedButton("Browse", icon=ft.Icons.FOLDER_OPEN, on_click=on_browse)
    load_btn = ft.FilledButton("Load", icon=ft.Icons.DOWNLOAD, on_click=on_load_click)

    controls = ft.Column(
        [
            ft.Row([ft.Icon(ft.Icons.AUTO_AWESOME_MOSAIC, color=ACCENT, size=28),
                    ft.Text("Shardset", size=26, weight=ft.FontWeight.BOLD)],
                   spacing=8),
            ft.Text("Rebuild an image out of geometric shards.",
                    size=13, color=MUTED),
            ft.Divider(),
            ft.Row([path_field]),
            ft.Row([browse_btn, load_btn, size_field]),
            ft.Divider(),
            mode_dd,
            count_label, count_slider,
            alpha_label, alpha_slider,
            ft.Row([repeat_field, workers_field]),
            animate_sw,
            cores_text,
            ft.Row([run_btn, stop_btn]),
            ft.Row([save_png_btn, save_svg_btn]),
            ft.Divider(),
            status,
        ],
        width=430,
        scroll=ft.ScrollMode.AUTO,
    )

    gallery = ft.Column(
        [
            progress_card,
            ft.Row(
                [
                    ft.Column([ft.Text("Original", weight=ft.FontWeight.BOLD),
                               orig_box], expand=True),
                    ft.Column([ft.Text("Reconstruction", weight=ft.FontWeight.BOLD),
                               result_box], expand=True),
                ],
                expand=True,
            ),
        ],
        expand=True,
    )

    page.add(ft.Row([controls, ft.VerticalDivider(), gallery], expand=True))


if __name__ == "__main__":
    ft.run(main)
