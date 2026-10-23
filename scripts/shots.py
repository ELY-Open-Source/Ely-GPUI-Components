"""Writes frontend/public/shots: native captures a browser cannot make."""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "frontend/public/shots"
GROUND = {"light": (252, 250, 247, 255), "dark": (19, 17, 16, 255)}

# Stories shown as a capture: their demo needs the desktop.
STORIES = [
    ("terminal", "terminal-terminaltabs-terminalsplit-terminaltoolbar-shellselector-terminalsearch-terminallink"),
    ("forms", "fileinput-dropzone"),
    ("forms", "pathinput"),
    ("files", "dragdropfiles"),
    ("theme", "themeimporter-syntaxthemes"),
    ("media", "imageupload"),
    ("shell", "trayicon-traymenu"),
    ("shell", "dockbadge"),
    ("shell", "windowdragregion"),
    ("shell", "windowresizeborder"),
]

# Live stories, and the windows their page scripts open.
WINDOWS = {
    "shell/splashscreen": ["shell-splash-window"],
    "shell/aboutdialog": ["shell-about-window"],
    "shell/multiwindow-manager": ["shell-managed-window"],
    "shell/miniwindow-compactmode-pictureinpicture": ["shell-mini-window", "shell-pip-window"],
    "shell/updatedialog": ["shell-update-window"],
    "shell/crashreporter": ["shell-crash-window"],
    "navigation/searchpalette-spotlightsearch-quicklauncher": ["navigation-launcher"],
    "theme/platform-style-iconthemeprovider-vibrancybackground": ["theme-frosted-window"],
}


def capture(args, into):
    """Runs the native capture; its window must stay in front."""
    subprocess.run(["cargo", "run", "-q", "-p", "ely-gallery", "--", *args, "--capture", str(into)], cwd=ROOT, check=True)


def trimmed(path):
    """The shot cut to its content, a margin around it."""
    im = Image.open(path).convert("RGB")
    # The title bar and the window's rounded edge stay out.
    edge, bar = 16, 112
    inner = im.crop((edge, bar, im.width - edge, im.height - edge))
    ground = Image.new("RGB", inner.size, inner.getpixel((inner.width // 2, inner.height - 8)))
    box = ImageChops.difference(inner, ground).convert("L").point(lambda v: 255 if v > 24 else 0).getbbox()
    if box is None:
        sys.exit(f"shots.py: {path} is blank")
    box = (box[0] + edge, box[1] + bar, box[2] + edge, box[3] + bar)
    pad = 48
    return im.crop((max(box[0] - pad, 0), max(box[1] - pad, 0), min(box[2] + pad, im.width), min(box[3] + pad, im.height)))


def grounded(path, mode):
    """A window's capture over the site's ground, its corners filled."""
    window = Image.open(path).convert("RGBA")
    ground = Image.new("RGBA", window.size, GROUND[mode])
    return Image.alpha_composite(ground, window).convert("RGB")


def save(im, name):
    im.save(OUT / f"{name}.jpg", quality=88, optimize=True)
    print(f"shots.py: {name}.jpg {im.width}x{im.height}")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    manifest = {"stories": [], "windows": WINDOWS}
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        for page, story in STORIES:
            into = tmp / f"{page}--{story}"
            capture(["--page", page, "--story", story], into)
            for mode in ("light", "dark"):
                save(trimmed(into / f"{page}-{mode}.png"), f"{page}--{story}-{mode}")
            manifest["stories"].append(f"{page}/{story}")
        for page in sorted({key.split("/")[0] for key in WINDOWS}):
            into = tmp / page
            capture(["--page", page], into)
            for name in (n for key, names in WINDOWS.items() if key.startswith(page + "/") for n in names):
                for mode in ("light", "dark"):
                    save(grounded(into / f"{name}-{mode}.png", mode), f"{name}-{mode}")
    (ROOT / "frontend/src/shots.json").write_text(json.dumps(manifest, indent=1) + "\n")


main()
