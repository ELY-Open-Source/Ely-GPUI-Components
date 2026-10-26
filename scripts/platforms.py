#!/usr/bin/env python3
"""Holds each component's `Platforms:` doc line to the cfgs in its code; an item without one runs everywhere."""
import re
import sys
from pathlib import Path

PLATFORMS = ("macOS", "Windows", "Linux", "web")
ALL = frozenset(PLATFORMS)
OS = {"macos": "macOS", "windows": "Windows", "linux": "Linux"}
FAMILY = {"wasm": {"web"}, "unix": {"macOS", "Linux"}, "windows": {"Windows"}}
# Items built everywhere that work only where a cfg in their file lets them; the line must name these.
RUNTIME = {
    ("crates/ely-core/src/buttons/share.rs", "ShareButton"): {"macOS"},
    ("crates/ely-core/src/forms/color/dropper.rs", "EyeDropper"): {"macOS"},
    ("crates/ely-misc/src/web.rs", "WebView"): {"macOS"},
    ("crates/ely-misc/src/web.rs", "IframeEmbed"): {"macOS"},
    ("crates/ely-desktop/src/native.rs", "TrayIcon"): {"macOS", "Windows", "Linux"},
    ("crates/ely-desktop/src/native.rs", "set_dock_badge"): {"macOS"},
    ("crates/ely-desktop/src/native.rs", "dock_badge"): {"macOS"},
}
# Files that build a RUNTIME item yet work everywhere, and why.
WRAPS = {
    "crates/ely-core/src/forms/color/picker.rs": "the eye dropper is one button of the picker, disabled where it cannot sample",
}
# Files whose platform cfgs choose how a thing works, never whether it does.
SAME_EVERYWHERE = {
    "crates/ely-core/src/theme/platform.rs": "the traffic lights sit at AppKit's button height on macOS and at the drawn height elsewhere",
    "crates/ely-terminal/src/colors.rs": "answers a native program's color query; the web has no pseudo-terminal to ask",
    "src/lib.rs": "binds the terminal's keys where the terminal builds; the terminal's items carry the line",
    "crates/ely-core/src/forms/text/actions.rs": "binds each platform's own keys for a field; the web's paste comes as the browser's event",
    "crates/ely-editor/src/keys.rs": "binds each platform's own keys for the editor; the web's paste comes as the browser's event",
    "crates/ely-documents/src/document/visual.rs": "binds Ctrl-Y for redo where it is the platform's key",
    "crates/ely-core/src/shell/chrome.rs": "a title bar's double-click does what the platform does, and macOS draws its own window buttons",
    "crates/ely-terminal/src/input.rs": "binds each platform's own copy, paste, clear and find keys",
    "crates/ely-terminal/src/pty.rs": "starts a Windows shell with the flags it takes",
    "crates/ely-core/src/typography/keys.rs": "spells `secondary` and the word key as the build's platform spells them",
}
DOC = re.compile(r"^\s*//[/!] Platforms: (.*)$")
KIND = re.compile(r"(pub(?:\([^)]*\))?\s+)?(?:(?:unsafe|async|const|extern\s+\S+)\s+)*(mod|use|struct|enum|union|fn|type|trait|const|static|impl|macro_rules!)\b\s*(\w*)")
CFG = re.compile(r"cfg!\(|#\[cfg(?:_attr)?\(")


def strip(text):
    """The text with comments, strings and char literals blanked, newlines kept."""
    out, i, n = [], 0, len(text)
    blank = lambda s: re.sub(r"[^\n]", " ", s)
    while i < n:
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            out.append(blank(text[i:j]))
            i = j
        elif text.startswith("/*", i):
            depth, j = 1, i + 2
            while depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            out.append(blank(text[i:j]))
            i = j
        elif (m := re.match(r'b?r(#*)"', text[i:])) and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            end = text.index('"' + m.group(1), i + m.end()) + 1 + len(m.group(1))
            out.append(blank(text[i:end]))
            i = end
        elif text[i] == '"' or text.startswith('b"', i) and not (i and (text[i - 1].isalnum() or text[i - 1] == "_")):
            j = i + (2 if text[i] == "b" else 1)
            while text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            out.append(blank(text[i:j + 1]))
            i = j + 1
        elif m := re.match(r"b?'(\\.[^']*|[^\\'])'", text[i:]):
            out.append(blank(m.group()))
            i += m.end()
        else:
            out.append(text[i])
            i += 1
    return "".join(out)


def closing(text, i):
    """The index just past the bracket that closes the one at `i`."""
    pairs, depth = {"(": ")", "[": "]", "{": "}"}, 0
    stack = []
    for j in range(i, len(text)):
        ch = text[j]
        if ch in pairs:
            stack.append(pairs[ch])
        elif stack and ch == stack[-1]:
            stack.pop()
            if not stack:
                return j + 1
    sys.exit(f"platforms: unclosed {text[i]} at {i}")


def split_top(s):
    """`s` cut at commas outside brackets."""
    parts, depth, start = [], 0, 0
    for k, ch in enumerate(s):
        depth += {"(": 1, "[": 1, "{": 1, ")": -1, "]": -1, "}": -1}.get(ch, 0)
        if ch == "," and depth == 0:
            parts.append(s[start:k])
            start = k + 1
    parts.append(s[start:])
    return [p.strip() for p in parts if p.strip()]


def holds(pred, where):
    """The platforms a cfg predicate holds on; features on, `test` off."""
    pred = pred.strip()
    if m := re.fullmatch(r"(all|any|not)\s*\((.*)\)", pred, re.S):
        args = [holds(a, where) for a in split_top(m.group(2))]
        if m.group(1) == "not":
            return ALL - args[0]
        if m.group(1) == "all":
            return frozenset.intersection(ALL, *args)
        return frozenset().union(*args)
    if m := re.fullmatch(r'(\w+)\s*=\s*"([^"]*)"', pred):
        key, value = m.groups()
        if key == "target_os" and value in OS:
            return frozenset({OS[value]})
        if key == "target_family" and value in FAMILY:
            return frozenset(FAMILY[value])
        if key == "feature":
            return ALL
    if pred in FAMILY:
        return frozenset(FAMILY[pred])
    if pred == "test":
        return frozenset()
    if pred == "debug_assertions":
        return ALL
    sys.exit(f"platforms: {where}: no rule for cfg({pred})")


def attr_gate(attr, where):
    """The platforms an outer attribute's body keeps an item to; ALL when it gates nothing."""
    if m := re.fullmatch(r"cfg\s*\((.*)\)", attr, re.S):
        return holds(m.group(1), where)
    if m := re.fullmatch(r"cfg_attr\s*\((.*)\)", attr, re.S):
        cond, *applied = split_top(m.group(1))
        inner = frozenset.intersection(ALL, *(attr_gate(a, where) for a in applied))
        return inner if inner == ALL else (ALL - holds(cond, where)) | inner
    return ALL


class Module:
    def __init__(self, path, name, parent, gate, public):
        self.path, self.name, self.parent, self.gate, self.public = path, name, parent, gate, public
        self.children, self.items, self.uses = {}, {}, []


def line_of(text, i):
    return text.count("\n", 0, i) + 1


def items_in(text, raw, start, end, where):
    """Each item between `start` and `end`: its attributes, kind match, header start, and body span."""
    i = start
    while i < end:
        while i < end and text[i].isspace():
            i += 1
        if i >= end:
            return
        attrs = []
        while text.startswith("#[", i) or text.startswith("#![", i):
            inner = text.find("[", i)
            close = closing(text, inner)
            if text[i + 1] != "!":
                attrs.append((i, raw[inner + 1:close - 1].strip()))
            i = close
            while i < end and text[i].isspace():
                i += 1
        if i >= end:
            return
        kind = KIND.match(text, i)
        head = i
        word = kind.group(2) if kind else None
        j = i
        while j < end:
            ch = text[j]
            if ch == ";":
                j += 1
                break
            if ch == "{" and word not in ("const", "static", "use", "type"):
                j = closing(text, j)
                while j < end and text[j].isspace():
                    j += 1
                j += j < end and text[j] == ";"
                break
            if ch in "([{":
                j = closing(text, j)
                continue
            j += 1
        yield attrs, kind, head, (head, j)
        i = j


def doc_start(raw, at):
    """The offset where the `///` lines just above `at`'s line begin, else `at`'s line."""
    start = raw.rfind("\n", 0, at) + 1
    while start > 0:
        above = raw.rfind("\n", 0, start - 1) + 1
        if not raw[above:start].strip().startswith("///"):
            break
        start = above
    return start


def module_path(path, name, is_root_like):
    base = path.parent if is_root_like else path.parent / path.stem
    for candidate in (base / f"{name}.rs", base / name / "mod.rs"):
        if candidate.is_file():
            return candidate
    sys.exit(f"platforms: {path}: no file for mod {name}")


class Crate:
    def __init__(self, root):
        self.files, self.explained, self.inactive = {}, {}, {}
        self.root = self.load(root, None, "crate", ALL, True)

    def text(self, path):
        if path not in self.files:
            raw = path.read_text(encoding="utf-8")
            self.files[path] = (raw, strip(raw))
            self.explained.setdefault(path, set())
            self.inactive.setdefault(path, [])
        return self.files[path]

    def load(self, path, parent, name, gate, public, span=None):
        raw, text = self.text(path)
        root_like = path.name in ("lib.rs", "mod.rs") or parent is None
        start, end = span if span else (0, len(text))
        opened = start
        while True:
            while start < end and text[start].isspace():
                start += 1
            if not text.startswith("#![", start):
                break
            close = closing(text, start + 2)
            body = raw[start + 3:close - 1].strip()
            gate &= attr_gate(body, f"{path}:{line_of(text, start)}")
            if re.search(r"target_os|target_family", body):
                self.explained[path].add(start)
            start = close
        module = Module(path, name, parent, gate, public)
        if not gate:
            self.inactive[path].append((opened, end))
            return module
        for attrs, kind, head, (a, b) in items_in(text, raw, start, end, path):
            where = f"{path}:{line_of(text, head)}"
            own = frozenset.intersection(ALL, *(attr_gate(t, where) for _, t in attrs))
            for at, t in attrs:
                if re.match(r"path\s*=", t):
                    sys.exit(f"platforms: {where}: no rule for #[path]")
                if re.search(r"target_os|target_family", t) and kind and kind.group(2) in ("mod", "use"):
                    self.explained[path].add(at)
            effective = gate & own
            if not effective:
                self.inactive[path].append((doc_start(raw, attrs[0][0] if attrs else head), b))
                continue
            if not kind:
                continue
            public_item, word, ident = bool(kind.group(1)) and kind.group(1).strip() == "pub", kind.group(2), kind.group(3)
            if word == "mod":
                body = text.find("{", head, b)
                if body < 0:
                    if span:
                        sys.exit(f"platforms: {where}: no rule for a file mod inside an inline mod")
                    child = self.load(module_path(path, ident, root_like), module, ident, effective, public_item)
                else:
                    child = self.load(path, module, ident, effective, public_item, (body + 1, closing(text, body) - 1))
                child.decl, child.decl_attrs = (path, line_of(text, head)), [at for at, _ in attrs]
                module.children.setdefault(ident, []).append(child)
            elif word == "use":
                module.uses.append((text[head:b], effective, where, public_item))
            elif word == "impl":
                self.check_impl(text, path, head, b, effective, own)
            elif word == "macro_rules!" or not ident:
                continue
            else:
                defs = module.items.setdefault(ident, [])
                defs.append({"path": path, "line": line_of(text, head), "top": line_of(text, attrs[0][0] if attrs else head), "gate": effective, "public": public_item, "attrs": [at for at, _ in attrs], "span": (a, b)})
        return module

    def check_impl(self, text, path, head, end, effective, impl_gate):
        raw = self.files[path][0]
        body = text.find("{", head, end)
        for attrs, kind, h, (_, b) in items_in(text, raw, body + 1, closing(text, body) - 1, path):
            gate = frozenset.intersection(impl_gate, *(attr_gate(t, f"{path}:{line_of(text, h)}") for _, t in attrs))
            if not gate & effective:
                self.inactive[path].append((doc_start(raw, attrs[0][0] if attrs else h), b))
                continue
            if kind and kind.group(1) and kind.group(1).strip() == "pub" and gate != ALL:
                sys.exit(f"platforms: {path}:{line_of(text, h)}: no rule for a public method under a platform cfg")

    def resolve(self, module, segments, gate=ALL, hops=0, skip=frozenset()):
        """Every (node, name or None, gate) a path may name from `module`, through any `use`; empty when it leaves the crate."""
        if hops > 16:
            sys.exit(f"platforms: {module.path}: a use chain past 16 hops at {'::'.join(segments)}")
        if not gate:
            return []
        if not segments:
            return [(module, None, gate)]
        head, rest = segments[0], segments[1:]
        if head == "crate":
            return self.resolve(self.root, rest, gate, hops, skip)
        if head == "self":
            return self.resolve(module, rest, gate, hops, skip)
        if head == "super":
            return self.resolve(module.parent, rest, gate, hops, skip)
        found = []
        for child in module.children.get(head, []):
            found += self.resolve(child, rest, gate & child.gate, hops, skip)
        if not rest and head in module.items:
            defined = gate & frozenset().union(*(d["gate"] for d in module.items[head]))
            if defined:
                found.append((module, head, defined))
        for k, (statement, use_gate, where, _) in enumerate(module.uses):
            if (id(module), k) in skip:
                continue
            for path, alias in use_paths(statement, where):
                if alias == head and path[-1] != "*":
                    found += self.resolve(module, path + rest, gate & use_gate, hops + 1, skip | {(id(module), k)})
        rest_gate = gate - frozenset().union(*(g for _, _, g in found))
        if not rest_gate:
            return found
        for k, (statement, use_gate, where, _) in enumerate(module.uses):
            if (id(module), k) in skip:
                continue
            for path, _ in use_paths(statement, where):
                if path[-1] == "*":
                    found += self.resolve(module, path[:-1] + segments, rest_gate & use_gate, hops + 1, skip | {(id(module), k)})
        return found

    def exports(self):
        """(module, name, definition's index) -> the platforms where a public path reaches that definition."""
        reach = {}

        def add(node, name, gate):
            for k, d in enumerate(node.items[name]):
                if d["public"] and gate & d["gate"]:
                    reach[(node, name, k)] = reach.get((node, name, k), frozenset()) | (gate & d["gate"])

        def walk(module, gate, seen):
            key = (id(module), gate)
            if not gate or key in seen:
                return
            seen.add(key)
            for name in module.items:
                add(module, name, gate)
            for children in module.children.values():
                for child in children:
                    if child.public:
                        walk(child, gate & child.gate, seen)
            for statement, use_gate, where, public in module.uses:
                if not public:
                    continue
                for path, _ in use_paths(statement, where):
                    glob = path[-1] == "*"
                    for node, name, reached in self.resolve(module, path[:-1] if glob else path, gate & use_gate):
                        if name is not None:
                            add(node, name, reached)
                        else:
                            walk(node, reached, seen)

        walk(self.root, ALL, set())
        return reach


def use_paths(statement, where):
    """(path, name it takes) for each path a `use` names."""
    body = re.sub(r"\s+", " ", statement).strip()
    body = re.sub(r"^(pub(\([^)]*\))? )?use ", "", body).rstrip(";").strip()

    def expand(prefix, tree):
        tree = tree.strip()
        if tree.endswith("}"):
            head, _, group = tree.partition("{")
            head = head.rstrip(": ")
            base = prefix + ([s for s in head.split("::") if s] if head else [])
            return [p for part in split_top(group[:-1]) for p in expand(base, part)]
        name, _, alias = tree.partition(" as ")
        path = prefix if name.strip() == "self" else prefix + name.strip().split("::")
        return [(path, alias.strip() or path[-1])]

    return expand([], body)


def library_roots():
    roots = [Path("src/lib.rs")] + [p / "src" / "lib.rs" for p in sorted(Path("crates").iterdir()) if p.name != "ely-gallery"]
    return [r for r in roots if r.is_file()]


def documented(raw_lines, top, line):
    """The `Platforms:` words in an item's doc, with their line, or None; `top` is its first attribute's line."""
    k = top - 2
    while k >= 0 and raw_lines[k].strip().startswith("///"):
        k -= 1
    for n in range(k + 1, line - 1):
        if m := DOC.match(raw_lines[n]):
            return m.group(1), n + 1
    return None


def parse(where, words):
    names = [w.strip() for w in words.rstrip(".").split(",")]
    if not words.endswith(".") or not names or [n for n in PLATFORMS if n in names] != names:
        sys.exit(f"platforms: {where}: write `Platforms: ` then some of {', '.join(PLATFORMS)}, in that order, and a full stop")
    return frozenset(names)


def named(platforms):
    return ", ".join(p for p in PLATFORMS if p in platforms) if platforms is not None else "nothing"


def main():
    faults, claimed, count = [], set(), 0
    crates = [Crate(root) for root in library_roots()]
    for crate in crates:
        for (module, name, k), reach in crate.exports().items():
            d = module.items[name][k]
            path = d["path"]
            runtime = RUNTIME.get((path.as_posix(), name))
            want = frozenset(runtime) if runtime is not None else (reach if reach != ALL else None)
            found = documented(crate.files[path][0].splitlines(), d["top"], d["line"])
            said = None
            if found:
                said = parse(f"{path}:{found[1]}", found[0])
                claimed.add((path, found[1]))
            if runtime is not None and reach != ALL:
                faults.append(f"{path}:{d['line']}: {name} is in RUNTIME yet cfgs already keep it to {named(reach)}")
            if said != want:
                why = f"RUNTIME keeps it to {named(want)}" if runtime is not None else f"its cfgs keep it to {named(want)}" if want else "nothing limits it"
                faults.append(f"{path}:{d['line']}: {name}: {why}; its doc names {named(said)}")
            if want is not None:
                count += 1
    for (rel, name), platforms in RUNTIME.items():
        raw, text = Path(rel).read_text(encoding="utf-8"), strip(Path(rel).read_text(encoding="utf-8"))
        drawn = set()
        for m in CFG.finditer(text):
            inner = raw[m.end():closing(text, m.end() - 1) - 1]
            if m.group() == "#[cfg_attr(":
                inner = split_top(inner)[0]
            if re.search(r"target_os|target_family", inner):
                drawn.add(holds(inner, rel))
        if not ({frozenset(platforms), ALL - frozenset(platforms)} & drawn):
            faults.append(f"{rel}: no cfg in it draws {named(platforms)}, as RUNTIME says of {name}")
    runtime_files = {rel for rel, _ in RUNTIME}
    names = {name for _, name in RUNTIME}
    for crate in crates:
        for path, (raw, text) in crate.files.items():
            rel = path.as_posix()
            starts = [0]
            for line in raw.splitlines(keepends=True):
                starts.append(starts[-1] + len(line))
            for k, line in enumerate(raw.splitlines()):
                if DOC.match(line) and (path, k + 1) not in claimed and not inactive(crate, path, text, raw, starts[k] + line.index("//")):
                    faults.append(f"{rel}:{k + 1}: a Platforms line on no public item")
            for m in re.finditer(r"\b(" + "|".join(sorted(names)) + r")::new\(", text):
                if inactive(crate, path, text, raw, m.start()):
                    continue
                owner = [n for (r, n) in RUNTIME if r == rel and owns(text, m.start(), n)]
                if rel not in WRAPS and not owner:
                    faults.append(f"{rel}:{line_of(text, m.start())}: builds {m.group(1)}; list what it builds in RUNTIME, or the file in WRAPS")
            loose = []
            for m in CFG.finditer(text):
                at = m.start()
                inner = raw[m.end():closing(text, m.end() - 1) - 1]
                if not re.search(r"target_os|target_family", inner):
                    continue
                if at in crate.explained[path] or gate_of_public(crate, path, at):
                    continue
                if rel not in runtime_files and rel not in SAME_EVERYWHERE and not inactive(crate, path, text, raw, at):
                    loose.append(line_of(text, at))
            if loose:
                faults.append(f"{rel}: platform cfgs at lines {', '.join(map(str, loose))} limit nothing public; list the item in RUNTIME or the file in SAME_EVERYWHERE")
    for rel in list(SAME_EVERYWHERE) + list(WRAPS) + sorted(runtime_files):
        if not Path(rel).is_file():
            faults.append(f"platforms.py names {rel}, which is gone")
    if faults:
        sys.exit("platforms:\n" + "\n".join(f"  {fault}" for fault in faults))
    print(f"platforms: {count} items name their platforms; every other item runs on {', '.join(PLATFORMS)}")


def owns(text, at, name):
    """Whether offset `at` lies in an `impl ... name` block, or `name`'s own body."""
    for m in re.finditer(r"\bimpl\b[^{;]*?\b" + name + r"\b[^{;]*\{", text):
        if m.start() < at < closing(text, m.end() - 1):
            return True
    return False


def gate_of_public(crate, path, at):
    """Whether the cfg at `at` is an attribute of a public item, whose reach the export check holds."""
    def visit(module):
        for defs in module.items.values():
            for d in defs:
                if d["path"] == path and at in d["attrs"] and d["public"]:
                    return True
        return any(visit(child) for children in module.children.values() for child in children)
    return visit(crate.root)


def inactive(crate, path, text, raw, at):
    """Whether `at` lies in code no platform builds: a module gated to none, or an item under such a cfg."""
    return any(a <= at < b for a, b in crate.inactive[path]) or under_test(text, raw, at)


def under_test(text, raw, at):
    """Whether `at` lies inside an item gated to no platform, a test module most often."""
    for m in re.finditer(r"#\[cfg\(", text):
        close = closing(text, m.end() - 1)
        if holds(raw[m.end():close - 1], "test scope"):
            continue
        j = close + 1
        while j < len(text) and text[j] != "{" and text[j] != ";":
            j += 1
        if j < len(text) and text[j] == "{" and m.start() <= at < closing(text, j):
            return True
    return False


if __name__ == "__main__":
    main()
