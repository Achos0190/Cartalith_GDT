"""Literal mutants for the 2026-11-23 vault connect fix (`shell/vault_saf_android.gd`).

Each mutant replaces one exact literal (it must occur once) in a SCRATCH copy of the Godot project, runs
`_vaultpadsaf_probe.gd`, and restores the file in `finally`. KILLED = the probe exits 1; SURVIVED = exit 0;
BUILD_ERROR = the script did not parse (exit 2 or a parse error), reported apart from SURVIVED.

    python -I mutants.py <godot 4.7.2 console exe> <scratch godot-project copy> <empty APPDATA dir>
"""
import os, subprocess, sys

MUTANTS = [
    ("five_arg_query", "return _resolver.query(uri, cols, null, null)",
     "return _resolver.query(uri, cols, null, null, null)"),
    ("get_by_index", "\tvar it = perms.iterator()\n\twhile it != null and bool(it.hasNext()):\n\t\tvar p = it.next()",
     "\tfor i in perms.size():\n\t\tvar p = perms.get(i)"),
]


def main() -> int:
    godot, proj, appdata = sys.argv[1:4]
    path = os.path.join(proj, "shell", "vault_saf_android.gd")
    orig = open(path, encoding="utf-8", newline="").read()
    bad = 0
    for name, old, new in MUTANTS:
        if orig.count(old) != 1:
            print(f"{name}: LITERAL_NOT_UNIQUE ({orig.count(old)})")
            bad += 1
            continue
        try:
            open(path, "w", encoding="utf-8", newline="").write(orig.replace(old, new))
            env = dict(os.environ, APPDATA=appdata)
            p = subprocess.run([godot, "--headless", "--path", proj, "--script", "res://_vaultpadsaf_probe.gd"],
                               capture_output=True, text=True, timeout=300, env=env)
            out = p.stdout + p.stderr
            if p.returncode == 2 or "Parse Error" in out:
                verdict = "BUILD_ERROR"
            else:
                verdict = "KILLED" if p.returncode == 1 else "SURVIVED"
            fails = [l.strip() for l in out.splitlines() if "FAIL:" in l]
            print(f"{name}: {verdict} {fails}")
            bad += verdict != "KILLED"
        finally:
            open(path, "w", encoding="utf-8", newline="").write(orig)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
