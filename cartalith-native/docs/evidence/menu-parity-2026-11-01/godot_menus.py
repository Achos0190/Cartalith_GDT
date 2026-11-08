#!/usr/bin/env python3
"""The Godot side of the menu parity check (2026-11-01).

Reads the shipped shell sources and the PC and mobile drafts, applies the drafts IN MEMORY (nothing is
written except godot_menus.json beside this script), and runs a small interpreter over the menu builders to
list every menu row, per platform, in the order Godot builds them, for two milestone states:

  today  PC-1, PC-4, PC-5 and PC-6 not built: each row's fallback form
  after  all four built: the host has every method in drafts/pc-shell/90_pc_milestone_stubs.gd, and
         context_tabs() lists inspect, note and ask

Platforms: desktop; tablet_below_1180dp (the shipped tablet bar); tablet_wide_1180dp_and_up (mobile draft 08);
phone_MENU (mobile draft 05's root, whose eight rows open the phone's own popups row for row, Ruling DX Q13).
Mobile draft 09 (Help ▸ Gesture reference… on the phone) is applied to menus.gd on every platform; it builds its
row on the phone only.

What it must never do: write anywhere but godot_menus.json, or pass a draft whose anchor moved. Each EDIT's old
text must occur exactly once, or the run stops.

What it is not: Godot. It interprets the subset of GDScript the menu builders use (rows, submenus, loops over
constant tables, `match`, conditions on platform and capability). A value only known at run time (recent
projects, GPU adapters, open windows) gives rows marked `generated`, carrying the expression that makes them.
Where a run-time list can be read from a source file it is (the engine's landmark vocabulary from
cartalith-civ, the quality tiers from render.rs). Every other assumption is in CONDITIONS and RUNTIME below,
and every condition it could not decide is listed under `unresolved` in the output.

Run:  python3 cartalith-native/docs/evidence/menu-parity-2026-11-01/godot_menus.py
"""
import ast
import glob
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
GP = os.path.join(REPO, "cartalith-native", "godot-project")
SHELL = os.path.join(GP, "shell")
PCD = os.path.join(REPO, "cartalith-native", "drafts", "pc-shell")
MSD = os.path.join(REPO, "cartalith-native", "drafts", "mobile-shell")
RUST = os.path.join(REPO, "cartalith-native", "crates")


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read().replace("\r\n", "\n")


# ── lexing ────────────────────────────────────────────────────────────────────────────────────────

def code_mask(line):
    """Per character: True where it is code, False inside a string literal ("..." or '...')."""
    mask, q, esc = [], None, False
    for ch in line:
        if q:
            mask.append(False)
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == q:
                q = None
        elif ch in "\"'":
            q = ch
            mask.append(False)
        else:
            mask.append(True)
    return mask


def strip_comment(line):
    mask = code_mask(line)
    for i, ch in enumerate(line):
        if mask[i] and ch == "#":
            return line[:i].rstrip()
    return line.rstrip()


def depth_delta(code):
    mask, d = code_mask(code), 0
    for i, ch in enumerate(code):
        if mask[i]:
            d += {"(": 1, "[": 1, "{": 1, ")": -1, "]": -1, "}": -1}.get(ch, 0)
    return d


def indent_of(line):
    n = 0.0
    for ch in line:
        if ch == "\t":
            n += 1
        elif ch == " ":
            n += 0.25
        else:
            break
    return n


def remove_lambdas(text):
    """Each inline lambda `func(...): body` becomes `None`, up to the `,` or `)` that ends it."""
    out = text
    while True:
        mask = code_mask(out)
        idx = next((m.start() for m in re.finditer(r"\bfunc\s*\(", out) if mask[m.start()]), -1)
        if idx < 0:
            return out
        j, depth = idx, 0
        while j < len(out):
            if mask[j]:
                ch = out[j]
                if ch in "([{":
                    depth += 1
                elif ch in ")]}":
                    if depth == 0:
                        break
                    depth -= 1
                elif ch == "," and depth == 0:
                    break
            j += 1
        out = out[:idx] + "None" + out[j:]


# ── source model ──────────────────────────────────────────────────────────────────────────────────

class Stmt:
    def __init__(self, text, indent):
        self.text, self.indent, self.children = text, indent, []


def logical_lines(raw):
    """Physical lines joined into statements: an open bracket or a trailing backslash continues one."""
    out, buf, ind, depth = [], None, 0, 0
    for line in raw:
        code = strip_comment(line)
        if not code.strip():
            continue
        if buf is None:
            buf, ind, depth = code.strip(), indent_of(line), depth_delta(code)
        else:
            buf += " " + code.strip()
            depth += depth_delta(code)
        if depth > 0 or buf.endswith("\\"):
            buf = buf[:-1].rstrip() if buf.endswith("\\") else buf
            continue
        out.append(Stmt(buf, ind))
        buf = None
    if buf is not None:
        out.append(Stmt(buf, ind))
    return out


def build_tree(stmts):
    root, stack = [], []
    for s in stmts:
        while stack and stack[-1].indent >= s.indent:
            stack.pop()
        (stack[-1].children if stack else root).append(s)
        if s.text.endswith(":"):
            stack.append(s)
    return root


def split_top(text, sep=","):
    parts, depth, cur, mask = [], 0, "", code_mask(text)
    for i, ch in enumerate(text):
        if mask[i] and ch in "([{":
            depth += 1
        elif mask[i] and ch in ")]}":
            depth -= 1
        if mask[i] and ch == sep and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur)
    return [p.strip() for p in parts]


DEF_RE = re.compile(r"^(static\s+)?func\s+(\w+)\s*\(")
SIG_END = re.compile(r"\)\s*(->\s*[\w\[\]. ]+\s*)?:\s*$")
CONST_RE = re.compile(r"^(?:static\s+)?const\s+(\w+)\s*(?::\s*[^=:]+?)?\s*:?=\s*(.*)$")
VAR_RE = re.compile(r"^(?:static\s+)?var\s+(\w+)\s*(?::\s*([^=]+?))?\s*(?::?=\s*(.*))?$")


def top_level_blocks(text):
    """[(kind, name, first_line, end_line)] for every column-0 func, const and var."""
    lines = text.split("\n")
    blocks, i = [], 0
    while i < len(lines):
        code = strip_comment(lines[i])
        m = DEF_RE.match(code)
        if m:
            j, sig = i, code
            while not SIG_END.search(sig):
                j += 1
                sig += " " + strip_comment(lines[j]).strip()
            k = j + 1
            while k < len(lines) and (lines[k] == "" or lines[k][0] in " \t"):
                k += 1
            while k - 1 > j and not lines[k - 1].strip():
                k -= 1
            blocks.append(("func", m.group(2), i, k))
            i = k
            continue
        matched = False
        for kind, rx in (("const", CONST_RE), ("var", VAR_RE)):
            m = rx.match(code)
            if m:
                k, depth = i + 1, depth_delta(code)
                while depth > 0 and k < len(lines):
                    depth += depth_delta(strip_comment(lines[k]))
                    k += 1
                blocks.append((kind, m.group(1), i, k))
                i, matched = k, True
                break
        if not matched:
            i += 1
    return blocks


class Func:
    def __init__(self, name, params, body):
        self.name, self.params, self.body = name, params, body


class Source:
    def __init__(self, path, text):
        self.path, self.text = path, text
        self.funcs, self.consts, self.vars = {}, {}, {}
        m = re.search(r"^class_name\s+(\w+)", text, re.M)
        self.class_name = m.group(1) if m else None
        lines = text.split("\n")
        for kind, name, a, b in top_level_blocks(text):
            if kind == "func":
                head, j = [], a
                while True:
                    head.append(strip_comment(lines[j]).strip())
                    if SIG_END.search(" ".join(head)):
                        break
                    j += 1
                sig = " ".join(head)
                inner = sig[sig.index("(") + 1: sig.rindex(")")]
                params = []
                for p in split_top(inner):
                    pm = re.match(r"(\w+)\s*(?::\s*[^=]+?)?\s*(?::?=\s*(.+))?$", p)
                    if pm:
                        params.append((pm.group(1), pm.group(2)))
                self.funcs[name] = Func(name, params, build_tree(logical_lines(lines[j + 1:b])))
            else:
                txt = " ".join(strip_comment(l).strip() for l in lines[a:b])
                if kind == "const":
                    self.consts[name] = CONST_RE.match(txt).group(2).strip()
                else:
                    vm = VAR_RE.match(txt)
                    self.vars[name] = (vm.group(3) or "").strip() or None


# ── applying the drafts in memory ─────────────────────────────────────────────────────────────────

def apply_defs(target, draft):
    """Every column-0 func / const / var a draft defines replaces the target's of the same name (from its
    first line to its last), or is appended at the end. Returns (text, replaced names, added names)."""
    dl = draft.split("\n")
    replaced, added = [], []
    for kind, name, a, b in top_level_blocks(draft):
        body = dl[a:b]
        blocks = {(k, n): (x, y) for k, n, x, y in top_level_blocks(target)}
        tl = target.split("\n")
        if (kind, name) in blocks:
            x, y = blocks[(kind, name)]
            tl[x:y] = body
            replaced.append(name)
        else:
            tl += [""] + body
            added.append(name)
        target = "\n".join(tl)
    return target, replaced, added


def edit(text, old, new, label):
    n = text.count(old)
    if n == 0 and text.count(new) >= 1:
        # MS-4 (2026-11-08): the draft's EDIT is already in the shipped file (mobile-shell draft 09 was transplanted),
        # so it must not be applied twice. Any other count still stops the run.
        return text
    if n != 1:
        raise SystemExit("EDIT %s: its old text occurs %d times, want 1: %r" % (label, n, old[:90]))
    return text.replace(old, new)


T = "\t"
# The PC drafts' EDIT steps that change rows, transcribed from each draft's header (its quoted old and new
# text). The run stops if an old text does not occur exactly once, so a moved anchor cannot pass silently.
PC_EDITS = [
    ("02 W2", T + '_live(p, "Markdown vault…", ID_VAULT)',
     T + '_live(p, "Markdown vault…", ID_VAULT, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_V)'),
    ("02 W5", T + '_live(p, "Run pipeline", ID_WORLD_RUN_PIPELINE, KEY_MASK_CTRL | KEY_R)',
     T + '_live(p, "Generate world", ID_WORLD_RUN_PIPELINE, KEY_MASK_CTRL | KEY_R)'),
    # Ruling DY (PQ6): Generate world moves to File, directly after New world… (draft 02's EDIT W6), and File's
    # signpost names Records (EDIT W7). Transcribed 2026-11-01 from the draft's header, as the others were.
    ("02 W6", T + '_live(p, "New world…", ID_NEW_WORLD, KEY_MASK_CTRL | KEY_N)\n' + T + "_session_gate_row(p, p.item_count - 1)",
     T + '_live(p, "New world…", ID_NEW_WORLD, KEY_MASK_CTRL | KEY_N)\n' + T + "_session_gate_row(p, p.item_count - 1)\n"
     + T + "## Ruling DY (PQ6): Generate world, directly after New world…, unless this bar's World menu holds it.\n"
     + T + "_build_generate_world_row(p)"),
    ("02 W7", T + '## "World", not "Data", since SM-1 renamed the menu (Ruling CK).\n' + T + 'for line in ["imports live under World ▸ Import",',
     T + '## "Records" since Ruling DY renamed the menu (it was "Data" until SM-1, then "World", Ruling CK).\n' + T
     + 'for line in ["imports live under Records ▸ Import",'),
    ("03 E1", T + '_live(p, "Deselect", ID_DESELECT, KEY_MASK_CTRL | KEY_D)\n' + T + "p.set_item_tooltip(p.item_count - 1,\n"
     + T + T + '"Clears the settlement, label or icon selection in the active domain. Not the same as Escape, '
     'which puts the tool down and deliberately leaves the selection alone.")',
     T + '_live(p, "Deselect", ID_DESELECT, KEY_MASK_CTRL | KEY_D)\n' + T
     + "p.set_item_tooltip(p.item_count - 1, _deselect_tip())\n" + T + "_build_selection_rows(p)"),
    ("03 E2", T + '_live(p, "Find on map…", ID_FIND_ON_MAP, KEY_MASK_CTRL | KEY_F)',
     T + '_live(p, "Find on map…", ID_FIND_ON_MAP, KEY_MASK_CTRL | KEY_F)\n' + T + "_build_map_surface_rows(p)"),
    ("05 N1", T + "for rid in WIN_REGION_IDS:\n" + T + T + "p.add_check_item(String(WIN_REGION_LABELS[rid]), rid)\n"
     + T + T + "p.set_item_checked(p.item_count - 1, true)",
     T + "for rid in WIN_REGION_IDS:\n" + T + T + "p.add_check_item(_region_label(rid), rid)\n"
     + T + T + "p.set_item_checked(p.item_count - 1, true)\n" + T + T + "if rid == ID_WIN_RIGHT:\n"
     + T + T + T + "_build_context_panel_menu(p)"),
    ("06 A1", T + "## `AssetDB::add_item`/`raster::decode_png` are real and bound now\n" + T
     + "## (`as_import_item`) -- the window's own slot grid is where a target\n" + T
     + "## slot gets focused, so this opens straight to it rather than\n" + T
     + "## duplicating slot-selection at the menu level.\n" + T + '_live(p, "Import image…", ID_IMPORT_IMAGE)',
     T + "## PC-9: Import image… left this menu (06_menus_help_assets.gd, EDIT A1)."),
]


# Mobile draft 09's EDIT G1 (Ruling DX Q13): Help ▸ Gesture reference… on the phone, after Keyboard reference….
# Transcribed from the draft's header, where it is quoted with `##` and four spaces before each line; the run
# stops if the draft no longer quotes either text. (Draft 03's H1 was withdrawn by the same ruling: MENU ▸ Help
# no longer draws the screen that line closed.)
MOBILE_EDITS = [
    ("09 G1", T + "if full:\n" + T + T + "_build_help_shortcuts_row(p)\n" + T + '_live(p, "Credits & academic principles", ID_HELP_CREDITS)',
     T + "if full:\n" + T + T + "_build_help_shortcuts_row(p)\n" + T + "_build_gesture_reference_row(p)\n" + T
     + '_live(p, "Credits & academic principles", ID_HELP_CREDITS)'),
]


def quoted_in(draft, text):
    """Whether `draft` quotes `text` in its header, the way the drafts quote an EDIT: each line after `##` and
    a run of spaces, tabs written as four spaces each."""
    lines = [l.replace(T, "    ") for l in text.split("\n")]
    quoted = [re.sub(r"^##\s+", "", l) for l in draft.split("\n") if l.startswith("##")]
    n = len(lines)
    strip = [l.strip() for l in lines]
    for i in range(len(quoted) - n + 1):
        if [q.strip() for q in quoted[i:i + n]] == strip:
            return True
    return False


def build_sources():
    applied = []
    menus = read(os.path.join(SHELL, "menus.gd"))
    for name in ["01_menus_ids_helpers.gd", "02_menus_world.gd", "03_menus_edit.gd", "04_menus_story.gd",
                 "05_menus_window.gd", "06_menus_help_assets.gd"]:
        menus, rep, add = apply_defs(menus, read(os.path.join(PCD, name)))
        applied.append({"draft": "pc-shell/" + name, "replaced": rep, "added": add})
    for label, old, new in PC_EDITS:
        # PC-9 shipped (2026-11-07): drafts 01-09 are in `shell/` now, so an EDIT whose ADDED code lines
        # (comments aside) are all already in the file is recorded as shipped, not applied. "Added" matters:
        # several EDITs only insert after their old text, which therefore stays in the file, and applying them
        # again would draw the inserted row twice. An EDIT none of whose added lines is present is applied as
        # before, and its old text must still occur exactly once.
        old_lines = set(old.split(chr(10)))
        added = [l for l in new.split(chr(10))
                 if l.strip() and not l.strip().startswith("##") and l not in old_lines]
        # An EDIT that only deletes code (06 A1) is shipped when its old text is gone.
        if (added and all(l in menus for l in added)) or (not added and menus.count(old) == 0):
            applied.append({"edit": "pc-shell " + label, "state": "shipped"})
            continue
        menus = edit(menus, old, new, label)
        applied.append({"edit": "pc-shell " + label})
    menus, rep, add = apply_defs(menus, read(os.path.join(MSD, "08_menus_tablet_bar.gd")))
    applied.append({"draft": "mobile-shell/08_menus_tablet_bar.gd", "replaced": rep, "added": add})
    d09 = read(os.path.join(MSD, "09_menus_phone_rows.gd"))
    menus, rep, add = apply_defs(menus, d09)
    applied.append({"draft": "mobile-shell/09_menus_phone_rows.gd", "replaced": rep, "added": add})
    for label, old, new in MOBILE_EDITS:
        if not (quoted_in(d09, old) and quoted_in(d09, new)):
            raise SystemExit("EDIT %s: draft 09 no longer quotes this text" % label)
        menus = edit(menus, old, new, label)
        applied.append({"edit": "mobile-shell " + label})
    phone = read(os.path.join(SHELL, "phone_menu.gd"))
    phone, rep, add = apply_defs(phone, read(os.path.join(MSD, "05_phone_menu_root.gd")))
    applied.append({"draft": "mobile-shell/05_phone_menu_root.gd", "replaced": rep, "added": add})
    srcs = {"menus": Source("menus.gd", menus), "phone": Source("phone_menu.gd", phone)}
    classes = {"DccMenus": srcs["menus"], "PhoneMenu": srcs["phone"]}
    for path in sorted(glob.glob(os.path.join(SHELL, "*.gd")) + glob.glob(os.path.join(SHELL, "workspaces", "*.gd"))):
        if os.path.basename(path) in ("menus.gd", "phone_menu.gd"):
            continue
        s = Source(os.path.basename(path), read(path))
        if s.class_name:
            classes[s.class_name] = s
    return srcs, classes, applied


# ── run-time facts read from the Rust sources ─────────────────────────────────────────────────────

_RUST_TEXT = None


def rust_has(fn):
    """Whether the gdext crate defines `fn <fn>(`: what `_engine_has()` asks the loaded library."""
    global _RUST_TEXT
    if _RUST_TEXT is None:
        _RUST_TEXT = "\n".join(read(p) for p in glob.glob(os.path.join(RUST, "cartalith-godot", "src", "**", "*.rs"),
                                                          recursive=True))
    return re.search(r"\bfn\s+%s\s*\(" % re.escape(fn), _RUST_TEXT) is not None


def landmark_kinds():
    """`cartalith_civ::landmark::kinds()` as the bridge returns it: key, label, family (as_str) per kind."""
    text = read(os.path.join(RUST, "cartalith-civ", "src", "landmark.rs"))
    body = text[text.index("pub fn kinds()"):]
    body = body[:body.index("\n}\n")]
    as_str = dict(re.findall(r"LandmarkFamily::(\w+)\s*=>\s*\"([a-z_]+)\"", text))
    out = []
    for m in re.finditer(r"LandmarkKindSpec\s*\{([^}]*)\}", body):
        spec = m.group(1)
        f = re.search(r"key:\s*\"([^\"]+)\",\s*label:\s*\"([^\"]+)\",\s*family:\s*F::(\w+)", spec)
        b = re.search(r"buildable:\s*(true|false)", spec)
        if f:
            out.append({"key": f.group(1), "label": f.group(2), "family": as_str.get(f.group(3), f.group(3).lower()),
                        "buildable": (b.group(1) == "true") if b else True})
    return out


def quality_tier_names():
    """The engine's quality tier names (`list_quality_tiers`: `QualityTier::ALL` mapped through `name()`)."""
    text = read(os.path.join(RUST, "cartalith-godot", "src", "render.rs"))
    m = re.search(r"pub const ALL: \[QualityTier; (\d+)\] = \[([^\]]+)\]", text)
    variants = [v.strip().split("::")[-1] for v in m.group(2).split(",")]
    i = text.index("impl QualityTier")
    blk = text[i:i + 4000]
    names = dict(re.findall(r"QualityTier::(\w+)\s*=>\s*\"([^\"]+)\"", blk))
    return [names.get(v, v.lower()) for v in variants]


# ── values ────────────────────────────────────────────────────────────────────────────────────────

class Unknown:
    def __init__(self, desc):
        self.desc = desc


class Chord:
    def __init__(self, mods=(), key=None):
        self.mods, self.key = frozenset(mods), key

    def __or__(self, o):
        return Chord(self.mods | o.mods, self.key or o.key) if isinstance(o, Chord) else NotImplemented

    def __eq__(self, o):
        return isinstance(o, Chord) and self.mods == o.mods and self.key == o.key

    def __hash__(self):
        return hash((self.mods, self.key))

    def text(self):
        if not self.key or self.key == "NONE":
            return ""
        order = [m for m in ("Ctrl", "Shift", "Alt", "Meta") if m in self.mods]
        return "+".join(order + [{"LEFT": "←", "RIGHT": "→", "DELETE": "Delete"}.get(self.key, self.key)])


class V2:
    def __init__(self, x, y):
        self.x, self.y = x, y


class Row:
    def __init__(self, kind, label, rid):
        self.kind, self.label, self.id = kind, label, rid
        self.accel, self.disabled, self.tooltip, self.meta = "", False, "", None
        self.submenu_name, self.session, self.generated, self.todo = None, False, None, False


class Popup:
    def __init__(self, name=""):
        self.name, self.rows, self.children, self.generated_mode = name, [], [], None

    def add(self, kind, label, rid):
        r = Row(kind, label, rid if rid != -1 else len(self.rows))
        r.generated = self.generated_mode
        self.rows.append(r)
        return r


class Obj:
    def __init__(self, kind, **kw):
        self.kind = kind
        self.__dict__.update(kw)


class FuncRef:
    def __init__(self, src, name, bound=()):
        self.src, self.name, self.bound = src, name, tuple(bound)


class ClassRef:
    def __init__(self, name):
        self.name = name


class Return(Exception):
    def __init__(self, value):
        self.value = value


class Continue_(Exception):
    pass


class Break_(Exception):
    pass


# ── assumptions ───────────────────────────────────────────────────────────────────────────────────

# Conditions on run-time state, answered for a fresh install on a desktop-class machine, first run done,
# a world open, nothing selected, no generation running. Keyed by the condition's source text.
CONDITIONS = {
    "_gpu_devices.is_empty()": False,              # at least one adapter enumerated (the mockup draws one)
    "usage.is_empty()": True,                      # "Memory: not measured yet" until a GPU generation
    "recents.is_empty()": False,                   # recent projects exist (the mockup draws three)
    "not FileAccess.file_exists(path)": False,
    "name_s != \"\"": True,
    "seed_s != \"\" and seed_s != \"seed unread\"": True,
    "edited != \"\"": True,
    "FileAccess.file_exists(dir.path_join(\"README.md\"))": False,   # an installed build (the mockup's form)
    "lh_sig != null": False,
    "'viewport' in _host": False,
    "\"viewport\" in _host": False,
    "(n as Window).visible": True,
    "t == \"\"": False,
    "not w.is_embedded()": False,
    "n.is_empty()": False,
    "text != \"\"": True,
    "pack.has(\"total_items\")": True,
    "gpu != \"\"": True,
    "sub == \"\"": False,
    "out.ends_with(\"▸\")": False,
    "bool(d.get(\"software\", false))": False,
    # The rest decide only the WORDING of a readout or tooltip (a refresher's internals), never whether a
    # row exists. Answers for a fresh install with a world open:
    "st.is_empty()": False,                        # the atlas reports a status
    "freed > 0": False,
    "chunks == 1": False,
    "chunks <= 0": True,                           # nothing baked yet
    "deepest < 0": True,
    "total > 0": True,                             # the machine reports its RAM
    "total <= 0": False,
    "bytes <= 0": False,
    "bytes < 1073741824": False,
    "r.is_empty()": True,                          # 3D not tested yet on a fresh install
    "forced >= 0": False,
    "not DccSettings.has_lighting_default(key)": True,   # no lighting default stored
    "absf(float(steps[i]) - v) < absf(float(steps[best]) - v)": False,
    "live.has(k)": True,
    "gb <= 0.0": True,
    "s.begins_with(\"edited \")": False,
    "parts.size() <= 2": True,
    "base != \"\" and norm.begins_with(base)": False,
    "bool(d.get('compute', True))": True,
    "not alts.is_empty()": False,
    "active <= 0": False,                          # the worker pool has started
    "stored == 0 or stored == active": True,       # no stored thread count: "Running N workers now"
    "not est.is_empty()": True,                    # the bridge gives a VRAM estimate
    "int(est.get(\"gw\", 0)) <= 0": False,          # a world exists, so the grid is known
}
# Run-time values the builders read, with the value used. cores = 8 is the mockup's own machine
# ("8 workers now"), so the CPU worker-thread ladder is compared like for like.
RUNTIME = {"cpu_logical_core_count": 8}
# Calls into other classes that are interpreted (all others are opaque run-time values).
INTERPRETED_STATIC = {("DccUnits", "label")}
# Builder functions that are not run: they connect signals, repaint, or refresh state set on open.
SKIP = {"_apply_theme_mode", "_connect_session_gate", "_stamp_pref_values", "_track_gpu_pref_row",
        "_seed_layouts_once", "refresh_undo_rows", "_refresh_clipboard_rows", "_refresh_autosave_menu",
        "_refresh_reset_stage_menu", "_refresh_family_counts", "_refresh_landmark_menu",
        "_refresh_asset_pack_stats", "_refresh_story_menu", "_refresh_theme_menu", "_refresh_units_menu",
        "_refresh_handedness_menu", "_refresh_gpu_retry_row", "_refresh_relief_exag_menu", "_refresh_gpu_mode_menu",
        "_refresh_gpu_fallback_menu",
        "_refresh_tiled_lod_menu", "_refresh_lod_levels_menu",
        "_refresh_tile_size_menu", "_refresh_atlas_cap_menu", "_refresh_undo_budget_menu",
        "_refresh_quality_row", "_refresh_lod_debug_menu", "_refresh_viewshed_status", "_refresh_recent_worlds",
        "_refresh_open_windows"}
# Rows filled when their submenu OPENS rather than at build: run once after the build so the model shows
# what opening them shows.
OPEN_FILLERS = ["_refresh_recent_worlds", "_refresh_open_windows"]
MILESTONE_METHODS = {"select", "select_back", "select_forward", "current_faction", "context_tabs",
                     "open_context_tab", "pin_context_panel", "follow_selection"}


# ── interpreter ───────────────────────────────────────────────────────────────────────────────────

class Interp:
    def __init__(self, srcs, classes, platform, after):
        self.srcs, self.classes, self.platform, self.after = srcs, classes, platform, after
        self.unresolved, self.menus, self.shortcut_defaults = set(), [], {}
        self.const_cache = {}
        app = read(os.path.join(SHELL, "app.gd")) + read(os.path.join(SHELL, "dcc_shell.gd"))
        self.host_funcs = set(re.findall(r"^(?:static\s+)?func\s+(\w+)", app, re.M))
        self.bridge_funcs = set(re.findall(r"^(?:static\s+)?func\s+(\w+)", read(os.path.join(SHELL, "engine_bridge.gd")), re.M))
        self.members = {}
        for name, init in srcs["menus"].vars.items():
            self.members[name] = self.ev(init, {"__src__": srcs["menus"]}) if init else None
        self.members.update({"_shell": Obj("shell"), "_bridge": Obj("bridge"), "_host": Obj("host"),
                             "_gpu_devices": Unknown("the enumerated GPU adapters")})

    @property
    def phone(self):
        return self.platform == "phone"

    @property
    def tablet(self):
        return self.platform in ("tablet_narrow", "tablet_wide")

    # ── expressions
    def py_text(self, text):
        text = remove_lambdas(text)
        mask = code_mask(text)
        out, i = [], 0
        while i < len(text):
            if not mask[i]:
                out.append(text[i])
                i += 1
                continue
            prev = text[i - 1] if i else " "
            m = re.match(r"(true|false|null)\b", text[i:])
            if m and not (prev.isalnum() or prev == "_" or prev == "."):
                out.append({"true": "True", "false": "False", "null": "None"}[m.group(1)])
                i += len(m.group(1))
                continue
            m = re.match(r"\s+as\s+[A-Z][\w.]*(\[[\w.]+\])?", text[i:])
            if m:
                i += len(m.group(0))
                continue
            if text[i:i + 2] in ("&&", "||"):
                out.append(" and " if text[i] == "&" else " or ")
                i += 2
                continue
            out.append(text[i])
            i += 1
        return "".join(out).strip()

    def ev(self, text, env):
        try:
            node = ast.parse(self.py_text(text), mode="eval").body
        except SyntaxError:
            return Unknown(text)
        return self.ev_node(node, env)

    def ev_node(self, n, env):
        e = lambda x: self.ev_node(x, env)
        if isinstance(n, ast.Constant):
            return n.value
        if isinstance(n, ast.Name):
            return self.lookup(n.id, env)
        if isinstance(n, (ast.List, ast.Tuple)):
            return [e(x) for x in n.elts]
        if isinstance(n, ast.Dict):
            return {e(k): e(v) for k, v in zip(n.keys, n.values)}
        if isinstance(n, ast.IfExp):
            return e(n.body) if self.truth(e(n.test), ast.unparse(n.test)) else e(n.orelse)
        if isinstance(n, ast.BoolOp):
            x = None
            for v in n.values:
                x = e(v)
                if isinstance(x, Unknown):
                    return Unknown(ast.unparse(n))
                if isinstance(n.op, ast.And) and not self.truthy(x):
                    return x
                if isinstance(n.op, ast.Or) and self.truthy(x):
                    return x
            return x
        if isinstance(n, ast.UnaryOp):
            x = e(n.operand)
            if isinstance(x, Unknown):
                return Unknown(ast.unparse(n))
            return (not self.truthy(x)) if isinstance(n.op, ast.Not) else (-x if isinstance(n.op, ast.USub) else x)
        if isinstance(n, ast.BinOp):
            a, b = e(n.left), e(n.right)
            if isinstance(n.op, ast.Mod) and isinstance(a, str):
                args = b if isinstance(b, list) else [b]
                args = [("…" if isinstance(x, Unknown) else x) for x in args]
                fmt = a
                if any(x == "…" for x in args):
                    fmt = re.sub(r"%[-+ 0#]*\d*(\.\d+)?[dfi]", "%s", a)
                try:
                    return fmt % tuple(args)
                except (TypeError, ValueError):
                    return Unknown(ast.unparse(n))
            if isinstance(a, Unknown) or isinstance(b, Unknown):
                return Unknown(ast.unparse(n))
            try:
                if isinstance(n.op, ast.BitOr):
                    return a | b
                if isinstance(n.op, ast.Add):
                    return a + b
                if isinstance(n.op, ast.Sub):
                    return a - b
                if isinstance(n.op, ast.Mult):
                    return a * b
                if isinstance(n.op, ast.Div):
                    return a // b if isinstance(a, int) and isinstance(b, int) else a / b
                if isinstance(n.op, ast.Mod):
                    return a % b
                if isinstance(n.op, ast.LShift):
                    return a << b
            except TypeError:
                return Unknown(ast.unparse(n))
            return Unknown(ast.unparse(n))
        if isinstance(n, ast.Compare):
            left = e(n.left)
            for op, c in zip(n.ops, n.comparators):
                right = e(c)
                if isinstance(left, Unknown) or isinstance(right, Unknown):
                    return Unknown(ast.unparse(n))
                try:
                    ok = {ast.Eq: lambda: left == right, ast.NotEq: lambda: left != right,
                          ast.Lt: lambda: left < right, ast.LtE: lambda: left <= right,
                          ast.Gt: lambda: left > right, ast.GtE: lambda: left >= right,
                          ast.In: lambda: left in right, ast.NotIn: lambda: left not in right}[type(op)]()
                except TypeError:
                    return Unknown(ast.unparse(n))
                if not ok:
                    return False
                left = right
            return True
        if isinstance(n, ast.Subscript):
            base, idx = e(n.value), e(n.slice)
            if isinstance(base, Unknown) or isinstance(idx, Unknown):
                return Unknown(ast.unparse(n))
            try:
                return base[idx]
            except (KeyError, IndexError, TypeError):
                return Unknown(ast.unparse(n))
        if isinstance(n, ast.Attribute):
            return self.attr(e(n.value), n.attr, ast.unparse(n))
        if isinstance(n, ast.Call):
            args = [e(a) for a in n.args]
            if isinstance(n.func, ast.Attribute):
                return self.method(e(n.func.value), n.func.attr, args, ast.unparse(n))
            if isinstance(n.func, ast.Name):
                return self.call_name(n.func.id, args, env, ast.unparse(n))
        return Unknown(ast.unparse(n))

    def truthy(self, v):
        if isinstance(v, (Popup, Obj, FuncRef, ClassRef, Chord, V2)):
            return True
        return bool(v)

    def truth(self, v, text):
        if isinstance(v, Unknown):
            key = text.strip()
            if key in CONDITIONS:
                return CONDITIONS[key]
            self.unresolved.add(key)
            return False
        return self.truthy(v)

    def lookup(self, name, env):
        if name in env:
            return env[name]
        if name in self.members:
            return self.members[name]
        src = env.get("__src__")
        if src is not None and name in src.consts:
            return self.const(src, name)
        if name.startswith("KEY_MASK_"):
            return Chord({name[9:].replace("CMD_OR_CTRL", "CTRL").capitalize()})
        if name.startswith("KEY_"):
            return Chord((), name[4:])
        if src is not None and name in src.funcs:
            return FuncRef(src, name)
        if name in self.classes or name in ("PopupMenu", "DccTheme", "DccSettings", "PopoutHost", "DisplayServer",
                                             "Input", "OS", "Engine", "FileAccess", "Vector2", "ProjectSettings"):
            return ClassRef(name)
        if name.startswith("TYPE_"):
            return name
        return Unknown(name)

    def const(self, src, name):
        key = (src.path, name)
        if key not in self.const_cache:
            self.const_cache[key] = Unknown(name)
            self.const_cache[key] = self.ev(src.consts[name], {"__src__": src})
        return self.const_cache[key]

    def attr(self, base, name, text):
        if isinstance(base, ClassRef):
            cls = self.classes.get(base.name)
            if cls is not None and name in cls.consts:
                return self.const(cls, name)
            return Unknown(text)
        if isinstance(base, Popup):
            return {"item_count": len(base.rows), "name": base.name}.get(name, Obj("signal"))
        if isinstance(base, V2):
            return getattr(base, name, Unknown(text))
        if isinstance(base, dict):
            return base.get(name, Unknown(text))
        if isinstance(base, Obj):
            if base.kind == "bridge":
                if name == "gpu_api":
                    return rust_has("gpu_enumerate_devices")
                if name == "generating":
                    return False
                if name in ("has_world", "save_api"):
                    return True
                if name == "world_gen":
                    return Obj("world_gen")
            if base.kind == "mb" and name == "text":
                return base.title
        return Unknown(text)

    def call_name(self, name, args, env, text):
        a0 = args[0] if args else None
        if name in ("String", "str"):
            if isinstance(a0, Unknown):
                return a0
            if isinstance(a0, float) and a0 == int(a0):
                return str(int(a0))
            return "" if a0 is None else str(a0)
        if name in ("int", "float", "bool"):
            if isinstance(a0, (Unknown, Chord)) or (name != "bool" and not isinstance(a0, (int, float, str))):
                return a0
            try:
                return {"int": int, "float": float, "bool": bool}[name](a0)
            except (TypeError, ValueError):
                return Unknown(text)
        if name == "range":
            return Unknown(text) if any(isinstance(a, Unknown) for a in args) else list(range(*args))
        if name == "typeof":
            return "TYPE_DICTIONARY" if isinstance(a0, dict) else "TYPE_STRING" if isinstance(a0, str) else Unknown(text)
        if name in ("PackedStringArray", "Array"):
            return list(a0) if args and isinstance(a0, list) else []
        if name == "is_instance_valid":
            return True
        if name in ("Callable", "PopupMenu"):
            return None
        ref = self.lookup(name, env)
        if isinstance(ref, FuncRef):
            return self.invoke(ref, args)
        return Unknown(text)

    def invoke(self, ref, args):
        args = list(args) + list(ref.bound)
        if ref.src is None:
            return self.host_call(ref.name, args)
        native = getattr(self, "n_" + ref.name, None)
        if native is not None and ref.src in self.srcs.values():
            return native(*args)
        if ref.name in SKIP:
            return None
        f = ref.src.funcs[ref.name]
        env = {"__src__": ref.src}
        for i, (pname, default) in enumerate(f.params):
            env[pname] = args[i] if i < len(args) else (self.ev(default, env) if default is not None else None)
        try:
            self.exec_block(f.body, env)
        except Return as r:
            return r.value
        return None

    def method(self, base, name, args, text):
        if isinstance(base, FuncRef):
            if name == "bind":
                return FuncRef(base.src, base.name, base.bound + tuple(args))
            if name == "call":
                return self.invoke(base, args)
            return None
        if isinstance(base, ClassRef):
            return self.class_call(base.name, name, args, text)
        if isinstance(base, Popup):
            return self.popup_call(base, name, args, text)
        if isinstance(base, Obj):
            return self.obj_call(base, name, args, text)
        if isinstance(base, Unknown):
            return Unknown(text)
        if isinstance(base, str):
            fns = {"to_upper": lambda: base.upper(), "to_lower": lambda: base.lower(),
                   "strip_edges": lambda: base.strip(), "is_empty": lambda: base == "",
                   "capitalize": lambda: " ".join(w[:1].upper() + w[1:] for w in base.replace("_", " ").split()),
                   "replace": lambda: base.replace(args[0], args[1]), "begins_with": lambda: base.startswith(args[0]),
                   "ends_with": lambda: base.endswith(args[0]), "length": lambda: len(base),
                   "join": lambda: base.join(str(x) for x in args[0]),
                   "substr": lambda: base[args[0]: args[0] + args[1] if len(args) > 1 and args[1] >= 0 else None]}
            return fns[name]() if name in fns else Unknown(text)
        if isinstance(base, list):
            if name == "size":
                return len(base)
            if name == "is_empty":
                return not base
            if name == "has":
                return args[0] in base
            if name == "append":
                base.append(args[0])
                return None
            if name == "clear":
                base.clear()
                return None
            if name == "slice":
                return base[args[0]: args[1] if len(args) > 1 else None]
            if name == "duplicate":
                return list(base)
            return Unknown(text)
        if isinstance(base, dict):
            if name == "get":
                return base.get(args[0], args[1] if len(args) > 1 else None)
            if name == "has":
                return args[0] in base
            if name == "is_empty":
                return not base
            if name == "size":
                return len(base)
            if name == "keys":
                return list(base)
            return Unknown(text)
        return Unknown(text)

    def class_call(self, cls, name, args, text):
        if cls == "PopupMenu" and name == "new":
            return Popup()
        if cls == "DccTheme" and name in ("is_phone", "is_tablet", "is_touch", "layout"):
            if name == "layout":
                width = {"tablet_wide": 1366.0, "tablet_narrow": 1024.0}.get(self.platform)
                return {"dp": V2(width, 900.0)} if width else {}
            return {"is_phone": self.phone, "is_tablet": self.tablet, "is_touch": self.phone or self.tablet}[name]
        if cls == "PopoutHost" and name == "supported":
            return self.platform == "desktop"
        if cls == "DisplayServer" and name == "is_dark_mode_supported":
            return True
        if cls == "DccSettings":
            if name == "shortcut_binding":
                return args[2]
            if name == "theme_mode":
                return "dark"
            if name == "popout_enabled":
                return False
            if name == "layout_names":
                return [d["name"] for d in self.const(self.srcs["menus"], "SEED_LAYOUTS")]
            return Unknown(text)
        c = self.classes.get(cls)
        if c is not None and (cls, name) in INTERPRETED_STATIC and name in c.funcs:
            return self.invoke(FuncRef(c, name), args)
        return Unknown(text)

    def host_call(self, name, args):
        if name == "has_method":
            return args[0] in self.host_funcs or (self.after and args[0] in MILESTONE_METHODS)
        if name == "context_tabs":
            return ["inspect", "note", "ask"] if self.after else []
        if name == "is_phone":
            return self.phone
        if name == "is_region_shown":
            return True
        if name == "get":
            return ({} if self.after else None) if args and args[0] == "selection" else Unknown("host.get")
        return Unknown("host." + name)

    def obj_call(self, base, name, args, text):
        if base.kind == "host":
            if name == "has_method" or name in self.host_funcs or (self.after and name in MILESTONE_METHODS):
                return self.host_call(name, args)
            return Unknown(text)
        if base.kind == "bridge":
            if name == "has_method":
                return args[0] in self.bridge_funcs
            if name == "quality_tiers":
                return quality_tier_names()
            if name == "landmark_kinds":
                return landmark_kinds()
            if name in RUNTIME:
                return RUNTIME[name]
            return Unknown(text)
        if base.kind == "world_gen" and name == "has_method":
            return rust_has(args[0])
        if base.kind == "shell":
            if name == "add_menu":
                pop = Popup(args[0])
                self.menus.append((args[0], pop))
                if isinstance(args[1], FuncRef):
                    self.invoke(args[1], [pop])
                return Obj("mb", title=args[0], popup=pop)
            if name == "is_session_mode":
                return False
            return None if name == "style_popup" else Unknown(text)
        if base.kind == "mb" and name == "get_popup":
            return base.popup
        if base.kind == "lambda" and name == "call":
            env = dict(base.env)
            for pname, v in zip(base.params, args):
                env[pname] = v
            try:
                self.exec_block(base.body, env)
            except Return as r:
                return r.value
            return None
        return Unknown(text)

    def popup_call(self, p, name, args, text):
        def row(i):
            return p.rows[i] if isinstance(i, int) and 0 <= i < len(p.rows) else None
        if name in ("add_item", "add_check_item", "add_radio_check_item"):
            kind = {"add_item": "command", "add_check_item": "check", "add_radio_check_item": "radio"}[name]
            p.add(kind, args[0], args[1] if len(args) > 1 and isinstance(args[1], int) else -1)
            return None
        if name == "add_separator":
            label = args[0] if args else ""
            p.add("caption" if label else "separator", label, -1)
            return None
        if name == "add_submenu_item":
            r = p.add("submenu", args[0], args[2] if len(args) > 2 else -1)
            r.submenu_name = args[1]
            return None
        if name == "add_child":
            if isinstance(args[0], Popup):
                p.children.append(args[0])
            return None
        r = row(args[0]) if args else None
        if name == "set_item_disabled":
            if r is not None and isinstance(args[1], bool):
                r.disabled = args[1]
            return None
        if name == "set_item_tooltip":
            if r is not None and isinstance(args[1], str):
                r.tooltip = args[1]
            return None
        if name == "set_item_text":
            if r is not None and isinstance(args[1], str):
                r.label = args[1]
            return None
        if name == "set_item_metadata":
            if r is not None:
                r.meta = args[1]
            return None
        if name == "clear":
            p.rows, p.children = [], p.children
            return None
        if name == "get_item_index":
            return next((i for i, x in enumerate(p.rows) if x.id == args[0]), -1)
        if name == "remove_item":
            if r is not None:
                p.rows.remove(r)
            return None
        if name == "set_item_id":
            if r is not None and isinstance(args[1], int):
                r.id = args[1]
            return None
        if name == "is_item_separator":
            return r is not None and r.kind in ("separator", "caption")
        if name == "get_item_text":
            return r.label if r is not None else ""
        if name == "get_item_submenu":
            return (r.submenu_name or "") if r is not None else ""
        if name == "is_item_disabled":
            return bool(r and r.disabled)
        if name in ("set_item_checked", "set_item_icon", "set_item_accelerator"):
            return None
        return Unknown(text)

    # natives: helpers whose whole effect is fixed and simple
    def n__bind_accelerator(self, p, idx, rid, accel):
        self.shortcut_defaults[rid] = accel
        if isinstance(p, Popup) and isinstance(idx, int) and 0 <= idx < len(p.rows) and isinstance(accel, Chord):
            p.rows[idx].accel = accel.text()

    def n__session_gate_row(self, p, idx):
        if isinstance(p, Popup) and isinstance(idx, int) and 0 <= idx < len(p.rows):
            p.rows[idx].session = True

    def n__todo(self, p, text, why):
        if isinstance(p, Popup):
            r = p.add("command", text, -1)
            r.disabled, r.tooltip, r.todo = True, why if isinstance(why, str) else "", True

    def n_shortcut_default(self, rid):
        return self.shortcut_defaults.get(rid, Chord((), "NONE"))

    # ── statements
    def exec_block(self, stmts, env):
        i = 0
        while i < len(stmts):
            s, t = stmts[i], stmts[i].text
            if t.startswith("if ") and t.endswith(":"):
                chain, j = [s], i + 1
                while j < len(stmts) and (stmts[j].text.startswith("elif ") or stmts[j].text == "else:"):
                    chain.append(stmts[j])
                    j += 1
                for c in chain:
                    if c.text == "else:":
                        self.exec_block(c.children, env)
                        break
                    cond = c.text[c.text.index(" ") + 1:-1].strip()
                    if self.truth(self.ev(cond, env), cond):
                        self.exec_block(c.children, env)
                        break
                i = j
                continue
            if t.startswith("for ") and t.endswith(":"):
                m = re.match(r"for\s+(\w+)(?:\s*:\s*[\w\[\]]+)?\s+in\s+(.+):$", t)
                it = self.ev(m.group(2), env)
                if isinstance(it, int) and not isinstance(it, bool):
                    it = list(range(it))
                if isinstance(it, dict):
                    it = list(it.keys())
                if isinstance(it, Unknown):
                    self.generated_loop(s, m.group(1), m.group(2), env)
                else:
                    for v in list(it):
                        env[m.group(1)] = v
                        try:
                            self.exec_block(s.children, env)
                        except Continue_:
                            continue
                        except Break_:
                            break
                i += 1
                continue
            if t.startswith("match ") and t.endswith(":"):
                subject = self.ev(t[6:-1], env)
                for arm in s.children:
                    pats = arm.text[:-1] if arm.text.endswith(":") else arm.text
                    hit = False
                    for pat in split_top(pats):
                        if pat == "_" or (not isinstance(subject, Unknown) and self.ev(pat, env) == subject):
                            hit = True
                    if hit:
                        self.exec_block(arm.children, env)
                        break
                i += 1
                continue
            if t.startswith("while ") and t.endswith(":"):
                i += 1
                continue
            if t.endswith(":"):
                m = re.match(r"var\s+(\w+)\s*:?=\s*func\s*\(([^)]*)\)", t)
                if m:
                    env[m.group(1)] = Obj("lambda", body=s.children, env=env,
                                          params=[x.split(":")[0].split("=")[0].strip() for x in split_top(m.group(2))])
                i += 1
                continue
            self.exec_simple(t, env)
            i += 1

    def generated_loop(self, s, var, expr, env):
        """A loop over a run-time list: the body runs once with the variable unknown, and every row it adds
        is marked as generated by this loop."""
        pops = [v for v in list(env.values()) + list(self.members.values()) if isinstance(v, Popup)]
        before = {id(p): p.generated_mode for p in pops}
        for p in pops:
            p.generated_mode = p.generated_mode or "for %s in %s" % (var, expr)
        env[var] = Unknown(var)
        try:
            self.exec_block(s.children, env)
        except (Continue_, Break_):
            pass
        for p in pops:
            p.generated_mode = before[id(p)]

    def exec_simple(self, t, env):
        if t == "pass":
            return
        if t == "continue":
            raise Continue_()
        if t == "break":
            raise Break_()
        if t == "return" or t.startswith("return "):
            raise Return(self.ev(t[7:], env) if t != "return" else None)
        if re.search(r"\.(connect|emit|disconnect)\(", t) or t.startswith(("push_error(", "push_warning(", "print(")):
            return
        m = re.match(r"var\s+(\w+)\s*(?::\s*[^=]+?)?\s*(?::?=\s*(.+))?$", t)
        if m:
            env[m.group(1)] = self.ev(m.group(2), env) if m.group(2) else None
            return
        m = re.match(r"^([A-Za-z_][\w.]*(?:\[[^\]]+\])?)\s*(\+=|-=|=)(?!=)\s*(.+)$", t)
        if m and "(" not in m.group(1):
            lhs, op, val = m.group(1), m.group(2), self.ev(m.group(3), env)
            sm = re.match(r"^(\w+)\[(.+)\]$", lhs)
            if sm:
                cont, key = self.ev(sm.group(1), env), self.ev(sm.group(2), env)
                if isinstance(cont, (dict, list)) and not isinstance(key, Unknown):
                    try:
                        cont[key] = val
                    except (IndexError, TypeError):
                        pass
                return
            if "." in lhs:
                obj, attr = lhs.rsplit(".", 1)
                o = self.ev(obj, env)
                if isinstance(o, Popup) and attr == "name":
                    o.name = val
                return
            if op != "=":
                cur = env[lhs] if lhs in env else self.members.get(lhs)
                if isinstance(cur, Unknown):
                    val = cur
                elif isinstance(val, Unknown):
                    val = Unknown(lhs)
                else:
                    try:
                        val = cur + val if op == "+=" else cur - val
                    except TypeError:
                        val = Unknown(lhs)
            if lhs in env:
                env[lhs] = val
            else:
                self.members[lhs] = val
            return
        self.ev(t, env)


# ── output ────────────────────────────────────────────────────────────────────────────────────────

def row_out(r, pop, it):
    lbl, gen = r.label, r.generated
    if isinstance(lbl, Unknown):
        gen = gen or "expression"
        lbl = "{" + lbl.desc + "}"
    meta = r.meta if isinstance(r.meta, str) else None
    if meta and meta == it.const(it.srcs["menus"], "META_READOUT"):
        kind = "readout"
    elif meta and meta == it.const(it.srcs["menus"], "META_SIGNPOST"):
        kind = "signpost"
    elif r.todo:
        kind = "disabled (not built)"
    else:
        kind = r.kind
    o = {"label": lbl, "kind": kind}
    if kind == "disabled (not built)" and r.kind != "command":
        o["base_kind"] = r.kind
    if r.accel:
        o["accel"] = r.accel
    if kind == "disabled (not built)":
        o["reason"] = r.tooltip
    elif r.disabled and kind not in ("readout", "signpost"):
        o["disabled_at_build"] = True
        if r.tooltip:
            o["reason"] = r.tooltip
    if r.session:
        o["session_mode_gated"] = True
    if gen:
        o["generated"] = gen
    o["id"] = r.id
    if isinstance(r.label, str) and r.label in NOT_BUILDABLE and r.generated is None:
        o["open_text"] = "%s   not buildable" % r.label
    if r.kind == "submenu":
        sub = next((c for c in pop.children if c.name == r.submenu_name), None)
        o["node"] = r.submenu_name
        o["items"] = popup_out(sub, it) if sub is not None else []
        if sub is None:
            o["missing_submenu"] = True
    return o


# Landmark kinds the engine marks unbuildable; `_refresh_landmark_menu()` writes "<label>   not buildable" on their
# rows when the submenu opens (the rows stay enabled: every row opens CIVIL ▸ Landmarks at its family).
NOT_BUILDABLE = set()


def popup_out(p, it):
    return [row_out(r, p, it) for r in p.rows]


def run_bar(srcs, classes, platform, after):
    it = Interp(srcs, classes, platform, after)
    it.invoke(FuncRef(srcs["menus"], "build"), [Obj("shell"), Obj("bridge"), Obj("host")])
    for fn in OPEN_FILLERS:
        SKIP.discard(fn)
        try:
            it.invoke(FuncRef(srcs["menus"], fn), [])
        finally:
            SKIP.add(fn)
    return it


# ── the phone's MENU: phone_menu.gd's screens over the phone's popups ─────────────────────────────

class PhoneInterp(Interp):
    """Runs phone_menu.gd's screen builders. A screen is a list of rows {label, kind, ...}; kinds are the
    bar's, plus `head` (a band), `chips` (a radio submenu drawn inline; its options in `items`), `drill` (a
    row that opens a popup or a screen; its rows in `items`), `value`, `info`, `quick`, `status`, `missing`."""

    def __init__(self, srcs, classes, bar):
        super().__init__(srcs, classes, "phone", bar.after)
        self.bar = bar
        for name, init in srcs["phone"].vars.items():
            self.members[name] = self.ev(init, {"__src__": srcs["phone"]}) if init else None
        self.members.update({"_shell": Obj("shell"), "_bridge": Obj("bridge")})

    def screen(self, fname, *args):
        body = []
        self.invoke(FuncRef(self.srcs["phone"], fname), [body] + list(args))
        return body

    def out(self, r, pop):
        return row_out(r, pop, self.bar)

    def n__head(self, body, text):
        body.append({"label": str(text).upper(), "kind": "head"})

    def n__add(self, body, row):
        if isinstance(row, dict):
            body.append(row)

    def n__info(self, body, text):
        body.append({"label": text if isinstance(text, str) else "{info}", "kind": "info"})

    def n__row(self, title, sub="", badge=None, trail=None, action=None, disabled=False, glyph=None):
        return {"label": title if isinstance(title, str) else "{%s}" % getattr(title, "desc", title),
                "kind": "disabled" if disabled is True else "command"}

    def n__value_row(self, label, value):
        return {"label": label if isinstance(label, str) else "{value}", "kind": "value"}

    def n__note_row(self, label, value):
        return self.n__value_row(label, value)

    def n__missing_row(self, title, why):
        return {"label": str(title), "kind": "missing", "reason": why if isinstance(why, str) else ""}

    def n__fill_quick(self, body):
        body.append({"label": "QUICK (pinned commands)", "kind": "quick"})

    def n__fill_status(self, body):
        body.append({"label": "STATUS (world, state, last autosave)", "kind": "status"})

    def n__menu_buttons(self):
        return [Obj("mb", title=t, popup=p) for t, p in self.bar.menus]

    def n__menu_popup(self, title, refresh=True):
        return next((p for t, p in self.bar.menus if t == title), None)

    def n__menu_row(self, mb):
        return {"label": mb.title, "kind": "drill", "items": self.fill_popup(popup_out(mb.popup, self.bar))}

    def n__menu_root_row(self, mb):
        """Draft 05's `_menu_root_row()`: the menu's name, opening its popup row for row (`phone_model()` reads
        the rows; the root needs only the row)."""
        return {"label": mb.title, "kind": "drill"}

    def n__live_root_sub(self, target):
        return ""

    def n__busy_why(self):
        return ""

    def n__engine_idle(self):
        return Obj("bridge")

    def n__asset_family_fill(self, key):
        return {"filled": 0, "total": 0}

    def n__slot(self, key):
        return "engine"

    def n__root_badge(self, *a):
        return None

    n__chevron = n__glyph = n__badge = n__trail_label = n__push_screen = n__root_badge

    def fill_popup(self, rows):
        """`_fill_popup` as draft 05 replaces it (2026-11-01): every row in order; a captioned separator becomes
        a band wherever it sits; a plain one becomes an empty band, except before the first row; a submenu
        becomes a drill row holding its own rows. (The shipped one skipped every leading separator, captioned
        or not; this check found the captions it lost.)"""
        out, first = [], True
        for r in rows:
            if r["kind"] in ("separator", "caption"):
                captioned = r["kind"] == "caption" and (r["label"] or "").strip() != ""
                if first and not captioned:
                    continue
                out.append({"label": r["label"].upper(), "kind": "head"} if captioned
                           else {"label": "", "kind": "separator"})
                continue
            first = False
            out.append(self.phone_row(r))
        return out

    # what a drill or chip row keeps of the popup row it stands for: `_popup_row()` draws a disabled submenu row
    # disabled, with its reason, so its gate and state travel with it
    CARRIED = ("id", "accel", "disabled_at_build", "reason", "session_mode_gated")

    def phone_row(self, r):
        if r["kind"] == "submenu":
            o = {"label": r["label"], "kind": "drill", "node": r.get("node"),
                 "items": self.fill_popup(r.get("items", []))}
            o.update({k: r[k] for k in self.CARRIED if k in r})
            return o
        return dict(r)

    @staticmethod
    def chip(x):
        """A row `_expand_sub` draws as a chip: a radio that is not disabled."""
        return x["kind"] == "radio" and not x.get("disabled_at_build")

    def expandable(self, items, node):
        """`_expandable`: not in NO_EXPAND, no nested submenu, at least two enabled radios."""
        if node in self.const(self.srcs["phone"], "NO_EXPAND"):
            return False
        if any(x["kind"] == "submenu" for x in items):
            return False
        return sum(1 for x in items if self.chip(x)) >= 2

    def expand(self, body, r, node, label):
        """`_expand_sub`: a radio submenu as a chip row (its enabled radios), its other rows under it; any
        other submenu as a drill row."""
        items = r.get("items", [])
        if not self.expandable(items, node):
            body.append(self.phone_row(r))
            return
        chips = [dict(x) for x in items if self.chip(x)]
        below = [self.phone_row(x) for x in items if not self.chip(x) and x["kind"] not in ("separator", "caption")]
        for x in below:
            x["drawn_below_chips"] = True
        o = {"label": label, "kind": "chips", "node": node, "items": chips + below}
        o.update({k: r[k] for k in self.CARRIED if k in r})
        body.append(o)

    def find_sub(self, p, node):
        return next((i for i, r in enumerate(p.rows) if r.kind == "submenu" and r.submenu_name == node), -1)

    def n__expand_sub(self, body, p, node, label, drawn):
        i = self.find_sub(p, node)
        if i < 0:
            body.append({"label": label, "kind": "missing"})
            return
        drawn[i] = True
        self.expand(body, self.out(p.rows[i], p), node, label)

    def n__inline_sub(self, body, p, node, drawn):
        i = self.find_sub(p, node)
        if i < 0:
            body.append({"label": node, "kind": "missing"})
            return
        drawn[i] = True
        for x in self.out(p.rows[i], p).get("items", []):
            if x["kind"] != "separator":
                body.append(self.phone_row(x))

    def n__act(self, body, p, rid, drawn):
        for i, r in enumerate(p.rows):
            if r.kind not in ("separator", "caption") and r.id == rid:
                drawn[i] = True
                body.append(self.phone_row(self.out(r, p)))
                return
        body.append({"label": "Menu item %s" % rid, "kind": "missing"})

    def n__rest_of(self, body, p, drawn):
        for i, r in enumerate(p.rows):
            if i in drawn:
                continue
            ro = self.out(r, p)
            if ro["kind"] == "caption":
                if ro["label"].strip():
                    body.append({"label": ro["label"].upper(), "kind": "head"})
                continue
            if ro["kind"] == "separator":
                continue
            if ro["kind"] == "submenu":
                self.expand(body, ro, r.submenu_name, ro["label"])
                continue
            body.append(self.phone_row(ro))


def phone_model(srcs, classes, bar):
    """MENU on the phone: the root `_fill_root()` draws, and for each menu the rows its root row opens. Since Ruling
    DX Q13 (mobile draft 05) every root row pushes its menu's popup, so a menu's rows are `_fill_popup()` over that
    popup. A draft that routes a menu through a bespoke screen again (a `MENU_SCREENS` table) stops the run: the
    screens' order and extra rows were what Q13 retired, and this model no longer interprets them."""
    ph = PhoneInterp(srcs, classes, bar)
    if "MENU_SCREENS" in srcs["phone"].consts:
        raise SystemExit("phone_menu.gd defines MENU_SCREENS again: MENU routes a menu through a phone screen, "
                         "which this extractor no longer models (Ruling DX Q13). Model the screen first.")
    root = ph.screen("_fill_root")
    menus = {}
    for title, pop in bar.menus:
        menus[title] = {"screen": "the %s popup (_menu_root_row -> _fill_popup)" % title,
                        "rows": ph.fill_popup(popup_out(pop, bar))}
    root_out = [{"label": r.get("label"), "kind": r.get("kind")} for r in root]
    return root_out, menus, ph.unresolved


def count_rows(rows):
    return sum(1 + count_rows(r.get("items", [])) for r in rows)


def main():
    srcs, classes, applied = build_sources()
    NOT_BUILDABLE.update(k["label"] for k in landmark_kinds() if not k["buildable"])
    out = {"source": "shipped shell at HEAD, with drafts/pc-shell 01-06 (and their EDITs), drafts/mobile-shell 05, "
                     "08 and 09 (and 09's EDIT G1) applied in memory by godot_menus.py",
           "applied": applied, "conditions": CONDITIONS, "runtime": RUNTIME, "states": {}}
    for state, after in (("today", False), ("after", True)):
        st = {}
        for platform, pf in (("desktop", "desktop"), ("tablet_below_1180dp", "tablet_narrow"),
                             ("tablet_wide_1180dp_and_up", "tablet_wide"), ("phone_MENU", "phone")):
            bar = run_bar(srcs, classes, pf, after)
            if pf == "phone":
                root, menus, unres = phone_model(srcs, classes, bar)
                st[platform] = {"root": root, "menus": menus, "unresolved": sorted(bar.unresolved | unres)}
                n = sum(count_rows(v["rows"]) for v in menus.values())
            else:
                st[platform] = {"menus": [{"menu": t, "rows": popup_out(p, bar)} for t, p in bar.menus],
                                "unresolved": sorted(bar.unresolved)}
                n = sum(count_rows(m["rows"]) for m in st[platform]["menus"])
            print("%-6s %-27s rows %-5d unresolved %d" % (state, platform, n, len(st[platform]["unresolved"])))
            for u in st[platform]["unresolved"]:
                print("         unresolved: %s" % u)
        out["states"][state] = st
    with open(os.path.join(HERE, "godot_menus.json"), "w", encoding="utf-8") as f:
        json.dump(out, f, ensure_ascii=False, indent=1)
    print("wrote", os.path.relpath(os.path.join(HERE, "godot_menus.json"), REPO))


if __name__ == "__main__":
    main()
