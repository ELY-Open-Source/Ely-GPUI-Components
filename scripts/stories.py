"""Writes crates/ely-gallery/stories.json: each page's stories and the components they show."""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PAGES = ROOT / "crates/ely-gallery/src/pages"
OUT = ROOT / "crates/ely-gallery/stories.json"
CALL = re.compile(r"(?<![\w.])(?:(\w+)::)?(\w+)\s*\(")
CHAIN = re.compile(r"\s*\.\s*\w+\s*(?:::<[^>]*>\s*)?\(")

# What a story draws beyond its title's names.
SHOWN = {
    ("forms", "Form"): ["FormLabel", "FormError"],
    ("tables", "DataTable"): ["Sparkline"],
    ("chat", "PromptInput"): ["VoiceInputButton", "VoiceWaveform"],
    ("shell", "MiniWindow"): ["AlwaysOnTop"],
    ("settings", "SettingsLayout"): [
        "ThemeSelector", "AccentColorPicker", "FontSizeControl", "DensitySelector",
        "KeyboardShortcutsList", "ProxySettings", "PrivacySettings", "NotificationSettings",
        "StartupSettings", "StorageSettings", "ResetToDefault", "ImportExportSettings",
        "AdvancedSettings", "DeveloperMode Toggle",
    ],
    ("theme", "Surfaces"): ["ColorTokens"],
    ("theme", "Ink"): ["ColorTokens"],
    ("theme", "Signals"): ["ColorTokens"],
    ("theme", "Series"): ["ColorTokens"],
    ("theme", "Syntax"): ["ColorTokens"],
    ("theme", "Shape"): ["Elevation"],
    ("theme", "Motion"): ["Spring"],
    ("theme", "ThemeEditor"): ["ThemeProvider"],
    ("rendering", "RenderSurface"): [
        "ShaderView", "CameraTexture", "ZeroCopyTexture", "MetalSurface",
    ],
}


def strip(text):
    """Blanks comments and literals' insides, keeping every offset."""
    out, i, n = list(text), 0, len(text)

    def blank(a, b):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif text.startswith("/*", i):
            j = text.index("*/", i) + 2
            blank(i, j)
            i = j
        elif m := re.match(r'b?r(#*)"', text[i:]):
            end = '"' + m.group(1)
            j = text.index(end, i + m.end()) + len(end)
            blank(i + m.end(), j - len(end))
            i = j
        elif text[i] == '"':
            j = i + 1
            while text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            blank(i + 1, j)
            i = j + 1
        elif m := re.match(r"'(\\.[^']*|[^'\\])'", text[i:]):
            blank(i, i + m.end())
            i += m.end()
        else:
            i += 1
    return "".join(out)


def literal(text, at):
    """The string literal starting at `at`, decoded."""
    m = re.compile(r'"((?:[^"\\]|\\.)*)"', re.S).match(text, at)
    return json.loads('"' + re.sub(r"\\\n\s*", "", m.group(1)) + '"')


def functions(folder):
    """Every `fn` in a page's folder, with its body's span."""
    found = {}
    for file in sorted(folder.rglob("*.rs")):
        module = file.stem
        text = file.read_text(encoding="utf-8")
        bare = strip(text)
        for m in re.finditer(r"\bfn\s+(\w+)\b", bare):
            depth, j = 0, bare.index("{", m.end())
            if ";" in bare[m.end():j]:
                continue
            k = j
            while True:
                depth += {"{": 1, "}": -1}.get(bare[k], 0)
                if depth == 0:
                    break
                k += 1
            found[(module, m.group(1))] = (text, bare, j, k)
    return found


def close(bare, at):
    """The offset past the bracket that closes `at`."""
    depth = 0
    for k in range(at, len(bare)):
        depth += {"(": 1, "[": 1, "{": 1, ")": -1, "]": -1, "}": -1}.get(bare[k], 0)
        if depth == 0:
            return k + 1


def sections(folder):
    """Section titles in drawing order; stray or nested ones fail."""
    fns = functions(folder)
    order, seen, opens = [], set(), {}

    def calls(module, name):
        text, bare, start, end = fns[(module, name)]
        for m in CALL.finditer(bare, start, end):
            owner = m.group(1) or module
            yield m, (owner, m.group(2)), m.group(2) == "section" and m.group(1) is None

    def draws(fn):
        """Whether the fn opens a section, itself or deeper."""
        if fn not in opens:
            opens[fn] = False
            opens[fn] = any(own or (callee in fns and draws(callee)) for _, callee, own in calls(*fn))
        return opens[fn]

    def walk(module, name):
        if (module, name) in seen:
            return
        seen.add((module, name))
        text, bare, _, _ = fns[(module, name)]
        inside = 0
        for m, callee, own in calls(module, name):
            if m.start() < inside and (own or (callee in fns and draws(callee))):
                sys.exit(f"{folder.name}/{module}.rs: a section opens inside {order[-1][0]!r}")
            if own:
                at = m.end() + len(bare[m.end():]) - len(bare[m.end():].lstrip())
                if text[at] != '"':
                    sys.exit(f"{folder.name}/{module}.rs: a section whose title is not a literal")
                order.append((literal(text, at), module))
                inside = close(bare, m.end() - 1)
                while chained := CHAIN.match(bare, inside):
                    inside = close(bare, chained.end() - 1)
            elif callee in fns:
                walk(*callee)

    walk("mod", "render")
    reached = {title for title, _ in order}
    for (module, _), (text, bare, start, end) in fns.items():
        for m in re.finditer(r"(?<![\w.])section\s*\(\s*\"", bare[start:end]):
            title = literal(text, start + m.end() - 1)
            if title not in reached:
                sys.exit(f"{folder.name}/{module}.rs: render never reaches section {title!r}")
    return [title for title, _ in order]


def slug(title):
    """As the gallery's ui::slug: lowercase letters and digits, dashes between."""
    out = re.sub(r"[^a-z0-9]+", "-", "".join(c.lower() if c.isascii() else " " for c in title))
    return out.strip("-")


def entries():
    """Component names by key, and ticked entries with a home."""
    names, homed = {}, []
    for file in sorted((ROOT / "tasks").glob("ch*.md")):
        for line in file.read_text(encoding="utf-8").splitlines():
            if m := re.match(r"- \[([ x])\] (.+?)(?: — | → | \(T\d|$)", line):
                aliases = [re.sub(r"\s*\([^)]*\)", "", name).strip() for name in m.group(2).split(" / ")]
                for name in aliases:
                    names[key(name)] = name
                if m.group(1) == "x" and home(line, aliases):
                    homed.append(aliases)
    return names, homed


def home(line, aliases):
    """False for a pointer: `A → b::B` lives at B."""
    if " → " not in line:
        return True
    target = re.search(r" → `([\w:]+)", line)
    return bool(target) and any(key(part) in map(key, aliases) for part in target.group(1).split("::"))


def key(name):
    return re.sub(r"[^a-z0-9]", "", re.sub(r"\([^)]*\)", "", name).lower())


def components(title, extra, known):
    """The task lists' names for the title's and `extra`'s names."""
    names = []
    for part in re.sub(r"\s*\([^)]*\)", "", title).split(" · "):
        for name in part.split(" → ")[0].split(" / ") + extra:
            if key(name) in known and known[key(name)] not in names:
                names.append(known[key(name)])
    return names


def pages():
    listing = (PAGES / "mod.rs").read_text(encoding="utf-8")
    order = re.findall(r"^\s+(\w+)::PAGE,$", listing, re.M)
    known, homed = entries()
    for (page, lead), extra in SHOWN.items():
        for name in extra:
            if key(name) not in known:
                sys.exit(f"stories.py: {name} in {page}/{lead} is no entry in tasks/*.md")
    out, shown = [], dict(SHOWN)
    for module in order:
        folder = PAGES / module
        head = (folder / "mod.rs").read_text(encoding="utf-8")
        field = lambda name: re.search(rf"^\s+{name}: (.+),$", head, re.M).group(1)
        page = json.loads(field("slug"))
        titles = sections(folder)
        slugs = [slug(title) for title in titles]
        for twice in {s for s in slugs if slugs.count(s) > 1}:
            sys.exit(f"{module}: two stories answer to {twice}")
        stories = []
        for t, s in zip(titles, slugs):
            extra = shown.pop((page, re.split(r" · | / ", t)[0]), [])
            stories.append({"title": t, "slug": s, "components": components(t, extra, known)})
        out.append(
            {
                "number": int(field("number")),
                "slug": page,
                "title": json.loads(field("title")),
                "summary": json.loads(field("summary")),
                "stories": stories,
            }
        )
    if shown:
        sys.exit(f"stories.py: SHOWN names no story for {sorted(shown)}")
    named = {key(name) for page in out for story in page["stories"] for name in story["components"]}
    for aliases in homed:
        if not any(key(name) in named for name in aliases):
            sys.exit(f"no story shows {' / '.join(aliases)}: name it in a title or in SHOWN")
    return out


def main():
    rows = [json.dumps(page, ensure_ascii=False) for page in pages()]
    text = '{"pages": [\n' + ",\n".join(rows) + "\n]}\n"
    if sys.argv[1:] == ["--check"]:
        if OUT.read_text(encoding="utf-8") != text:
            sys.exit(f"{OUT.relative_to(ROOT)} is stale: run python3 scripts/stories.py")
    elif sys.argv[1:]:
        sys.exit("usage: python3 scripts/stories.py [--check]")
    else:
        OUT.write_text(text, encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()
