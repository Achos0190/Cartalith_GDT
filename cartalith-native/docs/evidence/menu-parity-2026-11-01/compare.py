#!/usr/bin/env python3
"""The menu parity comparison (2026-11-01): mockup_menus.json (the mockup's own menuTree(), per platform)
against godot_menus.json (godot_menus.py: the shipped shell plus the PC and mobile drafts), per platform,
per menu, per row in order. Writes REPORT.md beside this script; prints the totals.

Rows are aligned with difflib on a normalised label, level by level (a submenu's rows are aligned against the
other side's submenu of the same name). For each aligned pair it compares the label, the kind, the accelerator
(not on the phone, which draws none), the disabled state with its reason, and Session-mode gating. The Godot
side is compared in its "after" state (PC-1, PC-4, PC-5 and PC-6 built); the "today" form is shown beside it
wherever the two differ, so each row says which form matches the mockup.

On the phone, a radio submenu Godot draws as chips is compared by its options, not its shape (the mockup's
own phone drawing, `menuRows()`, also draws one as chips and drops its separators).

Every difference is classified by CLASSIFY below: (a) the mockup is wrong, (b) a draft was wrong and is
corrected, (c) an owner decision, plus three kinds that are not faults: `m` same once its milestone lands,
`s` the same in substance (drawing only), `n` a row of the mockup's design that no draft builds yet. An
unclassified difference is printed and makes the run exit 1, so a new difference cannot pass unread.

What it must never do: decide a difference by editing either side. It reads the two JSON files and writes
REPORT.md, nothing else.

Run:  python3 cartalith-native/docs/evidence/menu-parity-2026-11-01/compare.py
"""
import difflib
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MOCK_DOC = json.load(open(os.path.join(HERE, "mockup_menus.json"), encoding="utf-8"))
MOCK = MOCK_DOC["platforms"]
GOD = json.load(open(os.path.join(HERE, "godot_menus.json"), encoding="utf-8"))
PLATFORMS = ["desktop", "tablet_wide_1180dp_and_up", "tablet_below_1180dp", "phone_MENU"]


# ── normalising ───────────────────────────────────────────────────────────────────────────────────

# The phone screens' own row kinds, mapped onto the bar's: a band is a caption, a chip row or a drill row is a
# submenu, a value row is a readout.
PHONE_KIND = {"head": "caption", "chips": "submenu", "drill": "submenu", "value": "readout"}


def norm_kind(r):
    k = r["kind"]
    return PHONE_KIND.get(k, k)


def readout_name(label):
    """A readout's name: the text before its value ('Working set   1.4 GB' -> 'Working set', 'Running 8 workers
    now' -> 'Running', 'Current grid 2048×2048 needs about' -> 'Current grid'). A readout that is all value
    ('Following the engine') keeps its whole text, so a missing name shows as a label difference."""
    for sep in ("   ", ": ", " — "):
        if sep in label:
            label = label.split(sep)[0]
    m = re.match(r"^(.*?)\s*[\d…]", label)
    return m.group(1) if m and m.group(1).strip() else label


def norm_label(r):
    k = norm_kind(r)
    lbl = r.get("label") or ""
    if k == "readout":
        lbl = readout_name(lbl)
    lbl = lbl.replace("   destructive", "").replace("’", "'")
    # a value a refresher stamps after the name ("Placed by hand   3", "Feature icons   22/31") is not the label
    lbl = re.sub(r"\s{3}\d+(/\d+)?$", "", lbl)
    lbl = re.sub(r"\s+", " ", lbl.replace("…", "").replace("⧉ ", "")).strip().lower()
    return lbl


def key(r):
    k = norm_kind(r)
    if k == "separator":
        return "—"
    if r.get("generated"):
        return "<generated>"
    if k == "caption":
        return "#" + norm_label(r)
    return norm_label(r)


def show(r, phone=False):
    """One cell of the report: the row's label with its kind, key and state, as the menu draws it."""
    if r is None:
        return ""
    k = norm_kind(r)
    lbl = r.get("label") or ""
    if k == "separator":
        return "———"
    s = lbl
    if k == "caption":
        s = "**%s**" % lbl
    elif r.get("kind") == "chips":
        s = "[chips] %s" % lbl
    elif k not in ("command",):
        s = "[%s] %s" % ({"disabled (not built)": "not built", "disabled (DESKTOP ONLY)": "DESKTOP ONLY"}.get(k, k), lbl)
    if r.get("accel") and not phone:
        s += " · " + r["accel"]
    if r.get("disabled_at_build"):
        s += " · dark"
    if r.get("session_mode_gated"):
        s += " · SM"
    if r.get("generated"):
        s += " · run-time list"
    return s.replace("|", "\\|")


def same_kind(m, g):
    mk, gk = norm_kind(m), norm_kind(g)
    if mk == gk:
        return True
    # a disabled radio or command whose reason says it is not built is the mockup's `todo`
    if mk == "disabled (not built)" and g.get("disabled_at_build") and (g.get("reason") or "").startswith("Not built"):
        return True
    return False


def reason_same(a, b):
    a, b = (a or "").strip().rstrip("."), (b or "").strip().rstrip(".")
    if not a or not b:
        return a == b
    if a in b or b in a:
        return True
    return a.split()[:5] == b.split()[:5]


def diffs(m, g, phone, gated_parent=False):
    """The ways an aligned pair differs, as short phrases. A Godot row from a run-time list is compared only
    by kind family (its label is whatever the run brings)."""
    out = []
    if g.get("generated"):
        return []
    if norm_label(m) != norm_label(g):
        out.append("label")
    if not same_kind(m, g):
        out.append("kind %s/%s" % (norm_kind(m), norm_kind(g)))
    if not phone and (m.get("accel") or "") != (g.get("accel") or ""):
        out.append("key %s/%s" % (m.get("accel") or "none", g.get("accel") or "none"))
    md = norm_kind(m) in ("disabled (not built)", "disabled (DESKTOP ONLY)")
    gd = norm_kind(g) == "disabled (not built)" or bool(g.get("disabled_at_build"))
    if md != gd and same_kind(m, g):
        out.append("disabled %s/%s" % ("yes" if md else "no", "yes" if gd else "no"))
    if md and gd and norm_kind(m) == "disabled (not built)" and not reason_same(m.get("reason"), g.get("reason")):
        out.append("reason")
    if g.get("open_text") and norm_kind(m) == "disabled (not built)":
        out.append("not-buildable kind: Godot keeps the row and writes '%s' on open" % g["open_text"])
    if bool(m.get("session_mode_gated")) != bool(g.get("session_mode_gated")) and not gated_parent:
        out.append("session gate %s/%s" % ("yes" if m.get("session_mode_gated") else "no",
                                           "yes" if g.get("session_mode_gated") else "no"))
    return out


def godot_states_differ(t, g):
    """Whether Godot's row today and after its milestones differ in anything the report shows. Symmetric, unlike
    `diffs()`, which reads its first row as the mockup's."""
    fields = ("label", "kind", "accel", "disabled_at_build", "reason", "session_mode_gated", "generated")
    return any((t.get(f) or None) != (g.get(f) or None) for f in fields)


def chip_order(items):
    """A radio submenu as a chip row draws: its radios as chips, then its other rows, separators and bands
    dropped. Applied to the mockup's items when Godot draws the submenu as chips, which is also what the
    mockup's own phone `menuRows()` does (its `seg` row), so the comparison is of options, not shape."""
    return [x for x in items if x["kind"] == "radio"] + \
        [x for x in items if x["kind"] not in ("radio", "separator", "caption")]


# ── aligning ──────────────────────────────────────────────────────────────────────────────────────

def align(a, b):
    """Pairs (i, j) of a and b in order; None for a row with no partner. Generated rows on the Godot side
    absorb the mockup's sample rows of the same gap (not its submenus: a sample list drawn inline on one side
    and as a submenu on the other is a shape difference the report must show)."""
    ka, kb = [key(x) for x in a], [key(x) for x in b]
    sm = difflib.SequenceMatcher(None, ka, kb, autojunk=False)
    out = []
    for op, i1, i2, j1, j2 in sm.get_opcodes():
        if op == "equal":
            out += [(i1 + k, j1 + k) for k in range(i2 - i1)]
            continue
        ai, bj = list(range(i1, i2)), list(range(j1, j2))
        gen = [j for j in bj if b[j].get("generated")]
        if gen:
            used_a = [i for i in ai if norm_kind(a[i]) not in ("separator", "caption", "submenu")]
            for i in ai:
                out.append((i, gen[0]) if i in used_a else (i, None))
            for j in bj:
                if j not in gen or not used_a:
                    out.append((None, j))
            continue
        # pair by label similarity, else positionally when both sides hold the same number of rows
        pairs, taken = [], set()
        for i in ai:
            best, score = None, 0.0
            for j in bj:
                if j in taken:
                    continue
                s = difflib.SequenceMatcher(None, ka[i], kb[j]).ratio()
                if s > score:
                    best, score = j, s
            if best is not None and (score >= 0.6 or (len(ai) == len(bj) and norm_kind(a[i]) == norm_kind(b[best])
                                                     and norm_kind(a[i]) not in ("separator",))):
                pairs.append((i, best))
                taken.add(best)
            else:
                pairs.append((i, None))
        merged, pj = [], 0
        for i, j in pairs:
            if j is not None:
                while pj < len(bj) and bj[pj] < j:
                    if bj[pj] not in taken:
                        merged.append((None, bj[pj]))
                    pj += 1
                pj = max(pj, bj.index(j) + 1)
            merged.append((i, j))
        for j in bj:
            if j not in taken and (None, j) not in merged:
                merged.append((None, j))
        out += merged
    return out


def today_partner(after_rows, today_rows):
    """For each after-row index, the today row that is the same command: by id, else by position."""
    out = {}
    ids = {}
    for j, r in enumerate(today_rows):
        if isinstance(r.get("id"), int) and r.get("id", -1) >= 100:
            ids.setdefault(r["id"], j)
    for i, r in enumerate(after_rows):
        if isinstance(r.get("id"), int) and r.get("id", -1) >= 100 and r["id"] in ids:
            out[i] = ids[r["id"]]
    sm = difflib.SequenceMatcher(None, [key(x) for x in after_rows], [key(x) for x in today_rows], autojunk=False)
    for op, i1, i2, j1, j2 in sm.get_opcodes():
        if op in ("equal", "replace") and i2 - i1 == j2 - j1:
            for k in range(i2 - i1):
                out.setdefault(i1 + k, j1 + k)
    return out


# ── classification ────────────────────────────────────────────────────────────────────────────────

# First match wins. Fields: platform (regex), path (regex on "Menu ▸ … ▸ row"), status (regex), cls, id. The
# ids are explained in NOTES, which REPORT.md prints. Every rule was written from the run's own unclassified
# list and checked at the Godot symbol and the mockup line its note names.
CLASSIFY = [
    # m: the same once the PC milestone lands; today's fallback is the drafted `_todo` or the shipped label.
    {"id": "PC-1", "cls": "m", "status": r"^same after", "path": r"Edit ▸ (Previous|Next) selection$"},
    {"id": "PC-4", "cls": "m", "status": r"^same after",
     "path": r"(Story ▸ Political history of the selection|Window ▸ Context panel"
             r"|Window ▸ Context panel ▸ (Inspect|Pin to the current subject|Follow the selection))$"},
    {"id": "PC-5", "cls": "m", "status": r"^same after", "path": r"(Story ▸ Note for the selection|Context panel ▸ Note)$"},
    {"id": "PC-6", "cls": "m", "status": r"^same after",
     "path": r"(Story ▸ Ask the assistant about the selection|Context panel ▸ Ask \(assistant\))$"},
    # n: the mockup's design, no draft builds it yet.
    {"id": "N1", "cls": "n", "path": r"(TOUCH · NEW|Touch & gestures…)$"},
    {"id": "N2", "cls": "n", "platform": r"^tablet", "path": r"Help ▸ Gesture reference…$"},
    {"id": "N3", "cls": "n", "platform": r"^tablet", "path": r"Open panels in their own window$"},
    # a: the mockup is wrong.
    {"id": "A1", "cls": "a", "path": r"Landmark types ▸ [^▸]+ ▸ [^▸]+$", "status": r"not-buildable kind"},
    {"id": "A2", "cls": "a", "path": r"Preferences ▸ Try the GPU again$", "status": r"disabled no/yes"},
    {"id": "A3", "cls": "a", "path": r"(Lighting rig defaults ▸ Source|Atlas cache ▸ Store)$", "status": r"^differs: label$"},
    {"id": "A4", "cls": "a", "path": r"LOD levels ▸ LOD 0–0", "status": r"^differs: label$"},
    {"id": "A5", "cls": "a", "path": r"Workspace ▸ World$", "status": r"session gate no/yes"},
    {"id": "A6", "cls": "a", "path": r"Open windows ▸ Context panel$", "status": r"^Godot-only"},
    {"id": "A7", "cls": "a", "platform": r"^phone", "path": r"Open panels in their own window$"},
    {"id": "A8", "cls": "a", "platform": r"^tablet_below", "path": r"^☰ ▸ Keyboard shortcuts…$", "status": r"label"},
    {"id": "A9", "cls": "a", "platform": r"^tablet_below", "path": r"^☰ ▸ (Edit|Assets|Preferences|Help) ▸ ",
     "status": r"^mockup-only"},
    {"id": "A10", "cls": "a", "platform": r"^tablet_below", "path": r"^World ▸ (Generation pipeline|Sculpt)$",
     "status": r"session gate no/yes"},
    {"id": "A11", "cls": "a", "platform": r"^tablet_below", "path": r"^World ▸ Generate world$", "status": r"^mockup-only"},
    # s: the same in substance; the difference is drawing only.
    {"id": "S1", "cls": "s", "path": r"(Anti-aliasing · anisotropy|3D viewport defaults|Documentation)$",
     "status": r"^differs: reason$"},
    # (a separator inside a submenu that only one side has belongs to that submenu's own class, so S2 takes
    # only a separator at the level being compared)
    {"id": "S2", "cls": "s", "platform": r"^phone", "path": r"▸ ———$", "status": r"^mockup-only$"},
    # c: owner decisions.
    {"id": "K1", "cls": "c", "path": r"Edit ▸ (Context card|Cartwheel at the pointer)$", "status": r"^Godot-only"},
    {"id": "P2", "cls": "c", "platform": r"^phone", "path": r"^Window ▸ (Context panel|Right dock)"},
    {"id": "P1", "cls": "c", "platform": r"^phone", "path": r"^(File|Assets|World|Preferences|Help) ▸ ",
     "status": r"^(Godot-only|mockup-only)"},
]

CLASS_NAMES = {"a": "(a) mockup wrong", "b": "(b) draft corrected", "c": "(c) owner decision",
               "m": "same after its milestone", "s": "same in substance", "n": "not drafted"}


def classify(platform, path, status):
    for rule in CLASSIFY:
        if re.search(rule.get("platform", ".*"), platform) and re.search(rule.get("path", ".*"), path) \
                and re.search(rule.get("status", ".*"), status):
            return rule["cls"], rule["id"]
    return "?", ""


# ── the comparison ────────────────────────────────────────────────────────────────────────────────

def compare_menu(platform, menu, mrows, after, today, phone, out, depth=0, path="", gated=False):
    tp = today_partner(after, today)
    for i, j in align(mrows, after):
        m = mrows[i] if i is not None else None
        g = after[j] if j is not None else None
        t = today[tp[j]] if (j is not None and j in tp) else None
        label = (m or g).get("label") or "———"
        rpath = "%s ▸ %s" % (path or menu, label)
        if m is not None and g is not None:
            d = diffs(m, g, phone, gated)
            dt = diffs(m, t, phone, gated) if t is not None else ["absent today"]
            if not d and not dt and g.get("generated"):
                status = "same (run-time list)"
            elif not d and not dt:
                status = "same"
            elif not d:
                status = "same after the milestone; today: " + ", ".join(dt)
            else:
                status = "differs: " + ", ".join(d)
        elif m is not None:
            status = "mockup-only"
        else:
            status = "Godot-only"
        out.append({"depth": depth, "m": m, "g": g, "t": t if (t is not None and g is not None
                                                                 and godot_states_differ(t, g)) else None,
                    "status": status, "cls": "", "rule": "", "path": rpath})
        if m is not None and g is not None and (m.get("items") is not None or g.get("items") is not None):
            mitems = m.get("items") or []
            if phone and g.get("kind") == "chips":
                mitems = chip_order(mitems)
            compare_menu(platform, menu, mitems, g.get("items") or [],
                         (t or {}).get("items") or g.get("items") or [], phone, out, depth + 1, rpath,
                         gated or bool(m.get("session_mode_gated")) or bool(g.get("session_mode_gated")))
        elif m is not None and m.get("items"):
            one_side(m["items"], "m", out, depth + 1, rpath)
        elif g is not None and g.get("items"):
            one_side(g["items"], "g", out, depth + 1, rpath)


def one_side(rows, side, out, depth, path):
    """The rows of a submenu only one side has, at every depth, so every row of each side is counted once."""
    for x in rows:
        rpath = path + " ▸ " + (x.get("label") or "———")
        out.append({"depth": depth, "m": x if side == "m" else None, "g": x if side == "g" else None, "t": None,
                    "status": ("mockup-only" if side == "m" else "Godot-only") + " (inside)", "cls": "", "rule": "",
                    "path": rpath})
        if x.get("items"):
            one_side(x["items"], side, out, depth + 1, rpath)


def godot_menus(state, platform):
    st = GOD["states"][state][platform]
    if platform == "phone_MENU":
        return [(t, v["rows"]) for t, v in st["menus"].items()]
    return [(m["menu"], m["rows"]) for m in st["menus"]]


def mockup_menus(platform):
    if platform == "phone_MENU":
        return [(m["menu"], m["rows"]) for m in MOCK[platform]["menus"]]
    return [(m["menu"], m["rows"]) for m in MOCK[platform]]


def flatten_phone_bands(rows):
    """The phone's Story screen holds four band rows that each open the band: present them as the band's
    caption followed by its rows, which is how the mockup's Story menu draws them."""
    out = []
    for r in rows:
        if r.get("kind") == "drill" and r.get("items") is not None and not r.get("node") and r.get("label") in (
                "Timeline", "Journeys", "Conflicts & campaigns", "Notes"):
            band = {"Timeline": "TIME", "Journeys": "JOURNEYS", "Conflicts & campaigns": "CONFLICTS & CAMPAIGNS",
                    "Notes": "NOTES"}[r["label"]]
            out.append({"label": band, "kind": "caption", "phone_band_row": r["label"]})
            out += r["items"]
        else:
            out.append(r)
    return out


def phone_root():
    """MENU's root, both sides. The mockup's list is mockup_menus.json's `root`, plus the MENUS band its
    `rowsMenuTab()` draws between Quick and the eight menus (`H('MENUS', 'the desktop menu bar, same names,
    same order')`), which mockup_menus.js does not list. Godot's is draft 05's `_fill_root()`."""
    mroot = MOCK["phone_MENU"]["root"]
    m = [{"label": mroot[0], "kind": "quick"}, {"label": "MENUS", "kind": "caption"}] + \
        [{"label": x, "kind": "submenu"} for x in mroot[1:-1]] + [{"label": mroot[-1], "kind": "status"}]
    g = [dict(r, kind={"head": "caption", "drill": "submenu", "command": "submenu"}.get(r["kind"], r["kind"]))
         for r in GOD["states"]["after"]["phone_MENU"]["root"]]
    out = []
    for i, j in align(m, g):
        mr = m[i] if i is not None else None
        gr = g[j] if j is not None else None
        st = "same" if (mr and gr and key(mr) == key(gr) and norm_kind(mr) == norm_kind(gr)) else \
            ("mockup-only" if gr is None else ("Godot-only" if mr is None else "differs: label"))
        out.append({"depth": 0, "m": mr, "g": gr, "t": None, "status": st, "cls": "", "rule": "",
                    "path": "MENU root ▸ " + (mr or gr)["label"]})
    return out


def run():
    report = {}
    for platform in PLATFORMS:
        phone = platform == "phone_MENU"
        after = dict(godot_menus("after", platform))
        today = dict(godot_menus("today", platform))
        mm = mockup_menus(platform)
        titles = [t for t, _ in mm] + [t for t in after if t not in [x for x, _ in mm]]
        per_menu = {}
        if phone:
            per_menu["MENU root"] = phone_root()
        for title in titles:
            mrows = dict(mm).get(title)
            grows, trows = after.get(title), today.get(title)
            out = []
            if mrows is None:
                out.append({"depth": 0, "m": None, "g": {"label": title, "kind": "submenu"}, "t": None,
                            "status": "Godot-only (whole menu)", "cls": "", "rule": "", "path": title})
            elif grows is None:
                out.append({"depth": 0, "m": {"label": title, "kind": "submenu"}, "g": None, "t": None,
                            "status": "mockup-only (whole menu)", "cls": "", "rule": "", "path": title})
            else:
                if phone and title == "Story":
                    grows, trows = flatten_phone_bands(grows), flatten_phone_bands(trows)
                compare_menu(platform, title, mrows, grows, trows, phone, out)
            per_menu[title] = out
        for rows in per_menu.values():
            for r in rows:
                if bucket(r["status"]) != "same":
                    r["cls"], r["rule"] = classify(platform, r["path"], r["status"])
        report[platform] = per_menu
    return report


# ── report ────────────────────────────────────────────────────────────────────────────────────────

def bucket(status):
    if status in ("same", "same (run-time list)"):
        return "same"
    if status.startswith("same after"):
        return "same after its milestone"
    if status.startswith("mockup-only"):
        return "mockup-only"
    if status.startswith("Godot-only"):
        return "Godot-only"
    return "differs"


def totals_of(report):
    totals, unclassified = {}, []
    for platform, menus in report.items():
        t = {}
        seen_m, seen_g = set(), set()
        for title, rows in menus.items():
            for r in rows:
                b = bucket(r["status"])
                t[b] = t.get(b, 0) + 1
                t["rows"] = t.get("rows", 0) + 1
                # a run-time row on the Godot side pairs with every sample row of the mockup's in its gap, so
                # each side's rows are counted once each, by identity
                if r["m"] is not None:
                    seen_m.add(id(r["m"]))
                if r["g"] is not None:
                    seen_g.add(id(r["g"]))
                if b != "same":
                    c = r["cls"] or "?"
                    t["class " + c] = t.get("class " + c, 0) + 1
                    t["rule " + (r["rule"] or "?")] = t.get("rule " + (r["rule"] or "?"), 0) + 1
                    if c == "?":
                        unclassified.append((platform, r["path"], r["status"]))
        t["mockup rows"], t["Godot rows"] = len(seen_m), len(seen_g)
        totals[platform] = t
    return totals, unclassified


# The report's prose: what each rule id means, with its evidence and, for (a), the exact fix. Written once here
# so that re-running the script cannot drop it.
INTRO = """# Menu parity: Godot against the mobile-shell mockup (2026-11-01)

Owner, verbatim: *"check the godot version of all menu's against the mockups in the artifact. Per platform and
option if they are actually the same."*

Evidence only. It records what differs and why. It is not a status record: how far any milestone has got is
`cartalith-native/docs/STATUS.md`'s answer.

## What was compared

- **Mockup:** `mockup_menus.json`. `mockup_menus.js` evaluates the mockup's own `menuTree()`, `tabletOverflow()` and
  `tabletWorldMenu()` from `design/mobile-shell-2026-10-31/Shell.dc.html` (last changed in `fcc4b6f`). Re-run on
  2026-11-01, it reproduced the committed file exactly.
- **Godot:** `godot_menus.json`, written by `godot_menus.py`. It covers the shipped shell at `{head}`, with these
  drafts applied in memory: `drafts/pc-shell/` 01-06 and their EDITs; `drafts/mobile-shell/` 05 and 08; and mobile 03's
  EDIT H1. This session has no Godot binary, so `godot_menus.py` interprets the menu builders. Every assumption it makes
  is in its `CONDITIONS` and `RUNTIME` tables (a fresh install, 8 logical cores). The interpreter left **0** conditions
  unresolved on every platform, in both states.
- **Per platform:**
  - **desktop:** `build()`'s eight menus.
  - **tablet ≥ 1 180 dp:** draft 08's whole bar.
  - **tablet < 1 180 dp:** the shipped bar, `☰ · File · World · Story`.
  - **phone:** MENU, which is draft 05's root over `phone_menu.gd`'s screens and the phone's own popups.
- **Two Godot states:**
  - **today:** PC-1, PC-4, PC-5 and PC-6 not built, so each row takes its drafted fallback.
  - **after:** those four milestones built.

  Rows are compared in the after state. The *Godot today* column is filled only where today's form differs.
- **Per row:** label, kind, accelerator (not on the phone, which draws none), disabled state and its reason,
  Session-mode gate, and submenu contents. A run-time list is compared by its pattern. On the phone, a radio submenu
  drawn as chips is compared by its options, not its shape.

Commands behind every count below:

```
python3 cartalith-native/docs/evidence/menu-parity-2026-11-01/godot_menus.py   # rows per platform and state; unresolved 0
python3 cartalith-native/docs/evidence/menu-parity-2026-11-01/compare.py       # this file; exits 1 if a difference is unclassified
```
"""

NOTES = [
    ("(a) The mockup is wrong: the fixes", [
        ("A1", "Landmark types: the 18 kinds the engine cannot place.",
         "The two sides agree on which kinds can be placed: 49 kinds, 18 not buildable, the same set "
         "(the mockup's `LANDMARK_FAMILIES` `*` marks against `cartalith-civ`'s vocabulary, read by "
         "`godot_menus.py::landmark_kinds()`). They draw them differently. The mockup draws each as a `todo`, a "
         "disabled row. Godot keeps it as an enabled row that reads `<kind>   not buildable` "
         "(`menus.gd::_refresh_landmark_menu()`). Pressing it opens CIVIL ▸ Landmarks at its family, as every leaf does: "
         "`_build_landmark_family()`'s signpost reads *Rows here read state; they do not arm a type.* "
         "**Fix:** in `menuTree()`'s `lmFamilies`, map a starred kind to "
         "`{ t: 'live', label: k.slice(0, -1) + '   not buildable', act: 'node', arg: 'civil/landmarks' }`."),
        ("A2", "Preferences ▸ Try the GPU again is drawn enabled.",
         "Godot builds the row disabled. Its tooltip is `GPU_RETRY_NONE_TIP` (*Nothing to clear: no GPU readback has "
         "failed this session…*), and `_refresh_gpu_retry_row()` enables it only while a failure is on record. "
         "**Fix:** give the row a `liveReason()` rule that returns *Nothing to clear: no GPU readback has failed this "
         "session* unless the mock state records a failure. This replaces the toast."),
        ("A3", "Lighting rig defaults ▸ Source and Atlas cache ▸ Store have names Godot does not draw.",
         "Godot's rows are value only. `_refresh_lighting_menu()` writes `Following the engine`, `All four stored here` "
         "or `n of 4 stored here`. `_refresh_atlas_cache_menu()` writes `%d chunk%s · <size> · to LOD n` (or `empty`). "
         "**Fix:** `{ t: 'readout', label: 'Following the engine', v: '' }` and "
         "`{ t: 'readout', label: '85 chunks · 1.2 GB · to LOD 3', v: '' }`."),
        ("A4", "Tile size · LOD levels ▸ LOD levels: `LOD 0–0   1 tiles`.",
         "Godot writes the singular (`menus.gd`, `\"LOD 0–%d   %s tile%s\"`). **Fix:** `1 tile`."),
        ("A5", "Window ▸ Workspace ▸ World carries no Session-mode gate.",
         "Godot gates it: `_session_gate_row()` runs on every domain in `DccShell.SESSION_HIDDEN_DOMAINS`, and "
         "`SESSION_MODE_TIP` names the row. **Fix:** add `sm: true` to the `World` radio. It applies on every "
         "platform, ☰ ▸ Window and the phone's MENU ▸ Window included."),
        ("A6", "Window ▸ Open windows: the Context panel's row label.",
         "Both sides list the panel while it is shown, so the list is the same. The labels differ. The mockup's "
         "`openWindowItems()` writes `Context panel · <tab> · <subject>`. Draft `pc-shell/05` writes "
         "`CONTEXT_PANEL_LABEL` (`Context panel`). No PC-4 contract method returns the panel's current tab: "
         "`90_pc_milestone_stubs.gd` has `context_tabs()`, the tabs it has, and `open_context_tab()`. **Fix:** the "
         "label `Context panel`. If the richer label is wanted, PC-4's contract needs a current-tab query first."),
        ("A7", "Phone: Preferences ▸ Open panels in their own window, drawn disabled as DESKTOP ONLY.",
         "`POPOUT_WINDOWS_SCOPE.md` PW-5 says *the setting is absent on phone and disabled-with-reason on tablet*. "
         "Godot omits it on the phone (`PopoutHost.supported()`). **Fix:** `menuRows()` skips a `desktopOnly` item on "
         "the phone and keeps the DESKTOP ONLY row on the tablet (N3)."),
        ("A8", "Tablet < 1 180: ☰ ▸ `Keyboard shortcuts…`.",
         "The ☰ row and Help's row are one builder, `_build_help_shortcuts_row()`. Draft `pc-shell/06` titles it "
         "`Keyboard reference…`, which is also what the mockup's own Help says. **Fix:** in `tabletOverflow()`, "
         "`{ t: 'live', label: 'Keyboard reference…', act: 'win:shortcuts', arg: 'ref' }`."),
        ("A9", "Tablet < 1 180: ☰'s submenus repeat the rows ☰ promotes.",
         "Godot builds them with `full = false`: `_edit(p, false)`, `_assets(p, false)`, "
         "`_preferences(p, false, true)` and `_help(p, false)`, called from `_overflow()`. So each row exists once. "
         "Undo, Redo and Undo history, ⧉ Asset library, Landmark types, Units, Keyboard reference… and About are at ☰'s "
         "top. Reset generation parameters and Reset one stage are in World. **Fix:** in `tabletOverflow()`, filter "
         "those rows (and the separators that framed them) out of the Edit, Assets, Preferences and Help submenus."),
        ("A10", "Tablet < 1 180: World ▸ Generation pipeline and Sculpt carry no Session-mode gate.",
         "Godot gates both (`menus.gd::_world()`, `_session_gate_row(p, mode_pipe_idx)` and `mode_sculpt_idx`). "
         "**Fix:** add `sm: true` to both radios in `tabletWorldMenu()`."),
        ("A11", "Tablet < 1 180: World ▸ Generate world under Run pipeline.",
         "`tabletWorldMenu()` appends World's items whole, so the narrow World has two rows for one command: id 710, "
         "Ctrl+R, `ID_WORLD_RUN_PIPELINE`, which `_pc_world()` and `_world()` both use. Godot's narrow World is "
         "`_tablet_world()` = `_world()` + `_data()`, with no Generate world. **Fix:** "
         "`.concat(w.items.filter((x) => x.label !== 'Generate world'))`. One label for the command on every form "
         "factor is K2."),
    ]),
    ("(b) Drafts this check corrected", [
        ("B1", "`drafts/mobile-shell/08_menus_tablet_bar.gd`, `build()`.",
         "Its whole-bar World was `_tablet_world if DccTheme.is_tablet() else _data`. That dropped the PC drafts' "
         "Generate world from the desktop, and it repeated Edit's two Reset rows in the wide tablet's World. It is now "
         "`_pc_world` (Ruling DU Q7, *the eight menus whole*). The header lists where each narrow-World row lives "
         "instead; README row 8 adds verify steps. `pc-shell/02` EDIT W1 and its README row now say to skip W1 when "
         "08 is applied."),
        ("B2", "`drafts/mobile-shell/05_phone_menu_root.gd`.",
         "Three fixes. (1) `story-root` drew the old MORE section, which opened with a `Story` head and ended with an "
         "empty `World` head; `_fill_story_section()` is replaced. (2) The `MENU_SCREENS` comment claimed File's own "
         "order; it is corrected. (3) `_fill_popup()` dropped a leading captioned separator, which lost "
         "FOR THE TABLE, SOURCES and ACTIVE PACK on the phone; it is replaced, with only that test changed. "
         "README row 5 is updated."),
        ("B3", "`drafts/mobile-shell/03_dcc_shell_phone_tabs.gd`.",
         "Draft 03 renames the tabs (MS-2) and names the documents to correct with them, but not the two strings "
         "the app draws. "
         "EDIT H1 is added: MENU ▸ Help's closing line becomes *…reachable through MAP · DOMAIN · STORY · MENU.*, the "
         "mockup's own About wording. H2 flags the bottom-bar coach mark and the two probes that pin it; its wording "
         "is MS-2's to write. `check_symbols.sh` gains an anchor check for both strings, and README row 3 is updated."),
        ("B4", "`drafts/mobile-shell/check_symbols.sh`.",
         "The need list gains `_fill_popup _band _popup_row _clean STORY_ROWS`: 93 calls, up from 88. The anchor "
         "section is new."),
    ]),
    ("(c) Owner decisions", [
        ("K1", "PQ10: where the card's and the cartwheel's rows live.",
         "The drafts put Edit ▸ Context card (Shift+F10) and Cartwheel at the pointer (Q) after Find on map… on the "
         "desktop and both tablets. The mockup has neither row. *Default: as drafted.* If it is taken, add both rows "
         "to the mockup's Edit."),
        ("K2", "PQ11: one label for the regenerate command.",
         "The command is Generate world on the PC and the wide tablet, and Run pipeline on the narrow tablet; it is the "
         "same command, id 710. It bears on A11. *Default: keep two labels.*"),
        ("K3", "PQ12: Alt+← / Alt+→ while a text field has focus.",
         "It bears on Edit ▸ Previous / Next selection; the rows match. *Default: yes, as a browser does.*"),
        ("P1", "Phone: MENU's File, Assets, Preferences and Help open the phone screens, which reorder the rows and add some.",
         "Ruling DT Q2: *the eight desktop menus row for row, in the same order and under the same names, plus Quick "
         "and Status*. The mockup's MENU follows it, and its band reads *same names, same order*. Draft 05 keeps "
         "`_fill_project()`, `_fill_assets()`, `_fill_prefs()` and `_fill_help()`. These draw:\n"
         "  - File: Save, Save as, New world and Open project first; Recent worlds inline under a head; Autosave with "
         "Interval chips (`Interval`, not `Autosave interval`).\n"
         "  - Assets: the nine families and the PACK values first.\n"
         "  - Preferences: Theme and Units first.\n"
         "  - Help: Runtime, Engine and Gesture reference first, and the closing info line, which the mockup puts in "
         "the About window.\n"
         "  - The screens keep their own titles (`SCREEN_TITLES`), so MENU ▸ File opens a screen titled *Project*, "
         "Assets one titled *Asset library*, and Help one titled *Help & about*. The row comparison cannot see a "
         "title.\n\n"
         "  `_fill_world()` also appends an *On the phone* band (Civilization, Data manager, Travel library, Simulation) "
         "that DT Q2 does not list. *Default: DT Q2 as written.* Those menus would open their popups row for row, as "
         "Edit and Window already do. Each phone-only row then needs a named home first (`MISTAKES.md`, \"Delete a "
         "surface\"). The candidates:\n"
         "  - Runtime, Engine and the info line: the About window, as the mockup draws it.\n"
         "  - Families and PACK values: the asset library and Assets ▸ Asset pack.\n"
         "  - Civilization: the CIVIL sheet (MS-3).\n"
         "  - Data manager: World ▸ ⧉ Data manager, already one of World's rows.\n"
         "  - Travel library: Story ▸ Travel library….\n"
         "  - Simulation (`_fill_sim()`: year, speed, playback, layers): no home is named yet. The nearest is the "
         "STORY tab's TIME band (Ruling DT Q1)."),
        ("P2", "Phone: Window ▸ Context panel.",
         "The mockup draws the Context panel check and its submenu (Inspect, Note, Ask, Pin, Follow) on the phone. "
         "`pc-shell/05` keeps `Right dock` and no submenu there. It reads `PC_SHELL_PROPOSAL.md` §3.11 (*the phone gets "
         "the model, not the panel*). The same section also gives the phone *the menu rows*. *Default: as drafted.* If "
         "it is taken, the mockup hides both Context panel rows on the phone and draws `Right dock`."),
    ]),
    ("Same in substance (drawing only)", [
        ("S1", "Disabled-row reasons: Anti-aliasing · anisotropy, 3D viewport defaults, Help ▸ Documentation.",
         "The mockup's reasons are short forms of Godot's tooltips, and they make the same claim."),
        ("S2", "Phone: plain separators inside a menu screen.",
         "`_rest_of()` skips an unlabelled separator, and `_add()` draws a hairline between every row. The mockup's "
         "phone draws a gap. Captions are kept on both sides."),
    ]),
    ("Not drafted: the mockup's design, which no draft builds yet", [
        ("N1", "Preferences ▸ TOUCH · NEW ▸ Touch & gestures… (tablet and phone).",
         "This is the proposal's Touch & gestures window. It holds Ruling DU Q5's switch, the hold durations and, per "
         "`MOBILE_SHELL_PROPOSAL.md`, the UI scale. `MOBILE_SHELL_MS1_SPEC.md` §3.6 specifies UI scale as its own "
         "`Preferences ▸ UI scale` submenu on touch builds, and leaves the panel to later work. On the phone, Godot "
         "has `TOUCH ▸ Gesture reference`, the read-only half."),
        ("N2", "Tablet: Help ▸ Gesture reference….",
         "The gestures screen exists only in `phone_menu.gd` (`_fill_gestures()`)."),
        ("N3", "Tablet: Preferences ▸ Open panels in their own window, disabled (DESKTOP ONLY).",
         "PW-5 (*disabled-with-reason on tablet*) is unbuilt. Godot omits the row there today."),
    ]),
    ("Same after its milestone", [
        ("PC-1", "Edit ▸ Previous / Next selection.", "A `_todo` row today; live, with Alt+← / Alt+→, from PC-1."),
        ("PC-4", "Story ▸ Political history of the selection; Window ▸ Context panel (the region row) and its Inspect, "
                 "Pin and Follow rows.", "The shipped label or a `_todo` today; the mockup's row from PC-4."),
        ("PC-5", "Story ▸ Note for the selection; Window ▸ Context panel ▸ Note.", "As above, from PC-5."),
        ("PC-6", "Story ▸ Ask the assistant about the selection (Ctrl+J); Window ▸ Context panel ▸ Ask.",
         "As above, from PC-6."),
    ]),
]

PLATFORM_TITLES = {"desktop": "Desktop", "tablet_wide_1180dp_and_up": "Tablet, 1 180 dp and wider",
                   "tablet_below_1180dp": "Tablet, below 1 180 dp", "phone_MENU": "Phone, MENU"}
COLUMNS = ["mockup rows", "Godot rows", "rows", "same", "same after its milestone", "differs", "mockup-only", "Godot-only",
           "class a", "class c", "class m", "class s", "class n"]


def write_report(report, totals):
    head = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=HERE, capture_output=True,
                          text=True).stdout.strip() or "HEAD"
    L = [INTRO.replace("{head}", head)]
    L.append("## Totals\n")
    L.append("*mockup rows* and *Godot rows* count each side's rows once, at every depth (on the phone, the root's "
             "eleven included; the mockup's three separators inside rows Godot draws as chips are not compared). "
             "*rows* counts the tables' rows: a run-time Godot row paired with several of the mockup's sample rows "
             "counts once per sample. Each row that is not *same* counts once more, under its class. Class (b) has "
             "no rows: its fixes are already applied on the Godot side.\n")
    L.append("| Platform | " + " | ".join(c.replace("class ", "") for c in COLUMNS) + " |")
    L.append("|---|" + "---:|" * len(COLUMNS))
    for p in PLATFORMS:
        L.append("| %s | %s |" % (PLATFORM_TITLES[p], " | ".join(str(totals[p].get(c, 0)) for c in COLUMNS)))
    L.append("\nRows per rule, per platform:\n")
    order = [rid for _, items in NOTES for rid, _, _ in items]
    rules = sorted({k[5:] for p in PLATFORMS for k in totals[p] if k.startswith("rule ")},
                   key=lambda r: order.index(r) if r in order else len(order))
    L.append("| Rule | " + " | ".join(PLATFORM_TITLES[p] for p in PLATFORMS) + " |")
    L.append("|---|" + "---:|" * len(PLATFORMS))
    for r in rules:
        L.append("| %s | %s |" % (r, " | ".join(str(totals[p].get("rule " + r, 0)) for p in PLATFORMS)))
    L.append("")
    for title, items in NOTES:
        L.append("## %s\n" % title)
        for rid, head_, body in items:
            L.append("- **%s. %s** %s" % (rid, head_, body))
        L.append("")
    L.append("## The tables\n")
    L.append("Columns: the mockup's row; Godot's row with PC-1, PC-4, PC-5 and PC-6 built; Godot today, shown only "
             "where it differs; the status; the class and rule. `›` marks one level of submenu. Marks: `[kind]`; "
             "`· Ctrl+…` the accelerator; `· dark` disabled when built, as Godot builds it; `· SM` disabled in "
             "Session mode; `· run-time list` a row repeated at run time (its text is the expression that makes it).\n")
    for p in PLATFORMS:
        phone = p == "phone_MENU"
        L.append("### %s\n" % PLATFORM_TITLES[p])
        for title, rows in report[p].items():
            L.append("#### %s\n" % title)
            L.append("| # | Mockup | Godot | Godot today | Status | Class |")
            L.append("|---:|---|---|---|---|---|")
            for n, r in enumerate(rows, 1):
                ind = "› " * r["depth"]
                cls = ""
                if r["cls"]:
                    cls = {"a": "(a) ", "b": "(b) ", "c": "(c) ", "m": "m ", "s": "s ", "n": "n "}.get(r["cls"], "? ") + r["rule"]
                L.append("| %d | %s | %s | %s | %s | %s |" % (
                    n, (ind + show(r["m"], phone)) if r["m"] else "",
                    (ind + show(r["g"], phone)) if r["g"] else "",
                    (ind + show(r["t"], phone)) if r["t"] else "",
                    r["status"].replace("|", "\\|"), cls))
            L.append("")
    with open(os.path.join(HERE, "REPORT.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(L).rstrip() + "\n")


if __name__ == "__main__":
    rep = run()
    totals, unclassified = totals_of(rep)
    write_report(rep, totals)
    for p in PLATFORMS:
        print("%-27s %s" % (p, ", ".join("%s %d" % (c, totals[p].get(c, 0)) for c in COLUMNS)))
    for u in unclassified:
        print("UNCLASSIFIED  %-26s %s  ::  %s" % u)
    print("wrote", os.path.join(os.path.relpath(HERE, os.path.join(HERE, "..", "..", "..", "..")), "REPORT.md"))
    sys.exit(1 if unclassified else 0)
