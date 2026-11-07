# The 2026-11-01 audits of v2.72, filed

Three read-only audits of `html/Cartalith v2.72 DCC test.html` (commit `7bd3b49`), run beside stage 2 in the
stage-2 workflow, and the version plan written from them. They were moved here from the session's scratchpad on
2026-11-02 so that the evidence outlives the session. Nothing here is state: what has been built from them is
`cartalith-native/docs/STATUS.md`'s answer, and what is left is `OUTSTANDING_WORK.md`'s.

| Path | What it is |
|---|---|
| `PLAN.md` | the plan agent's version plan (v2.74 onward), with its owner questions Q1–Q14. Its paths are re-pointed here |
| `perf/report.json` | the performance audit: 13 findings, each with evidence, impact, effort, proposal and risk |
| `perf/scripts/`, `perf/results/` | its measuring scripts (`sun_probe.js`, `bench.js`, `lod_ctrl.js`, `slider.js` and the rest) and their raw results |
| `visual/report.json` | the visual audit: 31 findings |
| `visual/*.js`, `visual/lib.js`, `visual/fixes.css` | its capture scripts, the in-page contrast and truncation census (`lib.js`), and every proposed rule as injected into v2.72 (`fixes.css`; read `PLAN.md` §3 for its two corrections) |
| `visual/data/`, `visual/sheets/`, `visual/shots/` | its measurements, its 28 contact sheets, and the 54 screenshots `PLAN.md` cites |
| `features/report.json` | the Godot-features audit: 25 findings, the ranked shortlist and the do-not-port list |
| `features/probe_audit.js`, `probe_v272.json`, `ls_probe.js`, `v272_funcs.txt` | its probes, their results and v2.72's function index |

**Paths inside the three `report.json` files** still name the scratchpad they were written in
(`/tmp/claude-0/…/scratchpad/audit-perf/`, `audit-visual/`, `audit-features/`). Read each as this folder's `perf/`,
`visual/` and `features/`. The screenshots `PLAN.md` does not cite (about 200 of 257) were not kept; the sheets show
most of them.

## What the owner has since decided

`PLAN.md`'s order and several of its questions are answered; where it disagrees, these win.

- **Order (its Q1).** Owner, 2026-11-02: *"Fixes and feature ports first then stage 3."* So the fix versions and the
  four ports come before stage 3, not after it (Ruling EF, `LARGE_ITEM_RULINGS.md`).
- **Ruling EC** (2026-11-01) answered Q2 (the toll fix: Godot's `> 0`), Q4 (rounded corners, as the design), Q10(d)
  (a half-resolution preview while a look slider is dragged) and Q12 (all four ports: Export map image, Find runs
  commands, relations and political history, then Measure).
- Every other question takes the default `PLAN.md` states for it, until the owner answers. Ruling EF lists them.
