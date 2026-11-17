"""Bump Cartalith's build version before a build (owner, 2026-11-17).

Format: ``v.YY.MM.DD-NNN`` -- the build date, then the number of that day's
build (001, 002, ...), restarting at 001 on a new day. The current version
lives in ``godot-project/version.txt`` (shown in Help > About) and is mirrored
into the Android export preset: ``version/name`` gets the same string and
``version/code`` gets ``YYMMDDNNN`` as an integer, which only ever grows, so
``adb install -r`` over an older build is accepted (Android refuses a lower
code). Run it once per APK build, before the export; commit both files.

Never edits anything else; refuses (exit 1) when the stored version does not
parse, rather than guessing a number.
"""
import datetime, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent / "godot-project"
VERSION = ROOT / "version.txt"
PRESET = ROOT / "export_presets.cfg"

def main() -> int:
    today = datetime.date.today().strftime("%y.%m.%d")
    cur = VERSION.read_text(encoding="utf-8").strip()
    m = re.fullmatch(r"v\.(\d\d\.\d\d\.\d\d)-(\d{3})", cur)
    if not m:
        print(f"version.txt holds {cur!r}, not v.YY.MM.DD-NNN; refusing", file=sys.stderr)
        return 1
    n = int(m.group(2)) + 1 if m.group(1) == today else 1
    if n > 999:
        print("more than 999 builds today; refusing", file=sys.stderr)
        return 1
    new = f"v.{today}-{n:03d}"
    VERSION.write_text(new + "\n", encoding="utf-8", newline="\n")
    code = int(today.replace(".", "") + f"{n:03d}")
    text = PRESET.read_text(encoding="utf-8")
    text = re.sub(r'^version/code=.*$', f"version/code={code}", text, flags=re.M)
    text = re.sub(r'^version/name=.*$', f'version/name="{new}"', text, flags=re.M)
    PRESET.write_text(text, encoding="utf-8", newline="\n")
    print(new, code)
    return 0

if __name__ == "__main__":
    sys.exit(main())
