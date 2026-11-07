# v2.74 lane A (fix pass + toll fix) — progress

File: /home/user/Cartalith_GDT/html/Cartalith v2.74 DCC test.html — final lane-A bytes md5 725e0bba0b6124154e057042bf20b1b1
Started as a byte copy of v2.73 (3be75cd6…, commit dfa97d2); the main loop checkpointed mid-lane at 1ca3c8b.
Parse check (all four blocks): scratchpad/v274a/tools/check.sh — run after every edit (ALL BLOCKS PARSE on 725e0bba).

## Lane A is DONE (every item of PLAN.md §5 + F06). What lane B needs to know about the CSS
Every v2.74 change carries a "v2.74 (Vxx/Pxx)" comment AT THE RULE IT CHANGES (no separate v2.74 block). Rules lane A
touched or added, in file order (grep "v2.74" to list them):
- :root dark: --faint #7d8285 (V08); color-scheme:dark (V02). :root[data-theme=light]: --faint #676762, --accent #8b550d
  (washes deliberately keep the old hue 164,101,15 — accent on a wash2 built on #8b550d would be 4.41), color-scheme:light.
- body.assets-mode rule (v2.24): + not-phone `body.assets-mode .dock-left{display:flex}` and `.dock-left > :not(.dock-head):not(#assetsPanel){display:none}`,
  `body.assets-mode.ldock-float #assetLibrary{margin-left:44px}`.
- #assetsHeaderBtn.on,#settlementsHeaderBtn.on {background:var(--wash2);color:var(--accent);border-color:var(--accent)} (V04)
- .legend (.86, --muted) (V21); .busy z-index:30 (V11)
- light-theme place popup block (V12) after `.place-edit-popup select,…`
- .cv-close/.re-close/.cfm-close panel2/--border, hover accent edge+ink (V05)
- .route-editor-modal background var(--bg); .cfm-body panel + side borders (V06)
- .pill.on ink var(--accInk) (V09)
- #scaleBar .85 (V21); .layers-fab .lp-item.on ink var(--accInk) (V09)
- #creditsModal z-index:76 (V10)
- #onboard #suArchSeg 3-col grid, .su-actions bottom:-24px (V17)
- .subtab:hover… then: #stTableHead > span,#stSpacer > div > span{min-width:0;…} + #stTableHead 1px transparent side borders (V01)
- details.cat-acc>summary::before margin-right 8px (V25 arrow)
- .al-card .al-thumb:has(> .al-empty) well + .al-empty --dim (V07)
- theme layer after input[type=range]: font-family inherit / mono numbers+selects / text padding 5px 7px (V20);
  checkbox/radio accent-color (V02); global :focus-visible ring (V19)
- theme layer after .subtab.on{background:var(--wash)}: .subtab.on / .set-nav button.on border reset + one 2px side,
  phone set-nav underline (V03); not-phone .subtab{letter-spacing:.02em;padding:9px 2px;min-width:0;flex:1 1 auto} (V18)
- #mbExportPanel #exportGeoBtn button box (V13)
- not-phone #toolOpts: #sculptModeSeg order:6, .to-spacer flex 0 0 0, #sBrushKm max-width 11em,
  #sculptFeatureSeg button padding 4px 7px (V15)
- not-phone .map-tl .funnel-btn 32×32 (V22) after #layersBtn
- not-phone .status-bar .readout wrap/26px (V14)
NOTE for lane B (corners/V24): segmented/tab on-states (.seg button.on, #civPmBar tabs, the CIVIL tools seg) are still
the solid accent fill — V24's wash is lane B's. V03's .subtab.on/.set-nav rules are lane A's (one 2px side).
JS (lane A): _civBakeKey; _stUpdateVisible; _jpClaimedAt (+comments, 2 planner hints); _civRenderFactionList;
_civRenderFactionsWorldOverview; _civInkOn + _civBuildFactionPicker; boot P4 (BOOT_OCEAN, _bootRenderPending, Skip);
VERSION/title/verTag 2.74; _toolsBlockSync; _dskTlSync/_dskTlExpand; Freehand hint; _dskCtxSync hint (+markup);
_shellDockSync assets branch + AL_STAGE_MIN; _dskDockHeadSync; _carEnter/ExitAssetsMode; Find owner for #assetsPanel;
_civOpenPlacePopup maxHeight; _shellPlace / _refitViewportNow / _syncChromeTop / window resize listeners (V272);
_dskToolOptsSync's Assets hint; the route editor "(land)" note.

## FINAL-BYTES VERIFICATION (725e0bba, frozen copy in scratchpad/v274a/finalbytes)
- probe_v274fix 131/0; probe_desktopshell 154/4 (v2.73's same four); probe_mobileshell 279/0; census 0 of 2408;
  run.sh 1280/0 (third run; two earlier runs flaked one random-world assert each — pinned seeds 101/202/303: v2.73 and
  v2.74 print identical suite output); run_um 882/0 (block 4 untouched); hash_gen1 vs v2.73 ALL IDENTICAL.
- Lane A is complete. Lane B (corners, design alignment) may start from 725e0bba.

## Measured (final)
- probe_v274fix.js: v2.74 131/0 (bytes 436ca134; the last edit after it is a hint string), v2.73 39 passed / 92 failed.
- census_v274.js: v2.74 0 of 2408 enabled runs below AA (final bytes 725e0bba); v2.73 561 of 2494.
- probe_mobileshell 279/0; probe_desktopshell 154/4 (v2.73's same 4); RC 8 shell probes identical to v2.73.
- run.sh: 1280/0 (a1d832be), 1279/1 bathymetry flake (436ca134), 1279/1 "traced length sane" (725e0bba) — both
  random-world asserts; pinned-seed comparison v2.73 vs v2.74 running → scratchpad/v274a/pinned/summary.txt
- run_um 882/0 (a1d832be; block 4 never changed); hash_gen1 ALL IDENTICAL (a1d832be); final-bytes rerun in progress.
- P4 n=6 alone: longest load task 2697 [2325..2971] → 332 [313..557] ms; DCL 3123 → 858; hash after Skip equal.
- V272 n=5: Layout per crossing 9→8 (860, 1180), 11.5→9.5 (600), boot 16→14; _syncChromeTop boot 28→0 ms.
- F06: tools/f06effect.js (numbers in CHANGELOG).
- Screenshots: html/evidence/v2.74/ (13), looked at 7.

## Docs: html/CHANGELOG_v2.74.md, html/CLAUDE_v2.74.md, html/README.md (lane A parts written).

# v2.74 lane B (corners V26, design alignment V24/V25/V29/V30, the version's deliverables) — progress
Scratch: scratchpad/v274b/ (own RC harness in v274b/rc; baselines in v274b/base: lane A's 725e0bba and v2.73 3be75cd6).
- 16:37 started from 725e0bba (lane A final). Reading done (EF/EC, PLAN §0/2/3/5/8/10-F06, reports, fixes.css, lib.js,
  CLAUDE_v2.72/73/74, CHANGELOG_v2.74, README, RC CLAUDE/CHANGELOG heads, MISTAKES preflight, design radii, dcc_theme.gd).
- NEXT: baseline censuses on 725e0bba (truncs 1366/1600 both themes, control types), then the CSS.
- 17:40 CSS built (file md5 6c27b6c6…): V26 radius system in the theme layer (tokens --r-btn 16/24, --r-tile 14/20,
  --r-field 12, --r-menu 10, --r-pill 999; rows/tabs/rail squared; the !important reset gone; 66 stylesheet + 35 inline
  radius declarations decided in place); D-block "flat" comment corrected; V24 washed seg thumbs (::before chip; fill
  kept for [data-civtool] and #sculptFeatureSeg), dock seg mono 10/500/.08em + basis auto nowrap, dock .btnrow buttons
  mono uppercase, Bake/Un-finalize full width, sub-tabs mono 500; V25 nine-selector label rule (in D4); V29 strip
  chips + .pill.on washed + .pill-x (JS template) + Layers row washed; V30 heading rule (theme layer, 11px per design).
- NEXT: censuses (truncs/wraps/contrast) vs 725e0bba, then probes, shots, docs.
- 18:05 candidate bytes d0fe0443 (frozen in scratchpad/v274b/cand/): + grip tab corners (phone), Types-tab quarters line
  opacity removed (3.19 → hint ink), #suArchSeg 3n divider, #reParty seg padding. Census vs 725e0bba (both seeds, 1366
  and 1600): 0 new truncations outside the Settlements table; wraps listed (sculptPresetSeg, genSculpt btnrow as before,
  civSubGeneration btnrow at 1366). probe_fixpass.js written (lane A's 131 sections + lane B); first full run on
  d0fe0443: 214/4 — the 4 were the truncation compare flagging the Settlements table head (lane A's V01 ellipses);
  probe fixed (table excluded, keys lower-cased), re-running that section; v2.73 run in progress.
- NEXT: census_v274, mobile/desktop shell probes, RC shell probes, run.sh/run_um/hash_gen1, final shots, docs.
- 18:00 mobileshell on d0fe0443 failed 4 (DS-13 − at its corner; the tablet chevron's corners ×3): hit-testing
  follows border-radius, and 24px touch pills lost their corners as targets. FIX: touch --r-btn/--r-tile = 12 (the
  port's touch btn_radius; within EC's "about 10–16 px"), DS-13 round buttons keep a full-box ::after target.
  Corner hits re-measured all true (tools/hit.js). Probe touch assertions updated (12px + corner presses).
- 18:00 final verification started on 5604c9ea (scratchpad/v274b/final/): browser stream (fixpass v2.74 with v2.73,
  fixpass v2.73, census, mobileshell, desktopshell, RC 8) and node stream (run.sh 1280/0 ✓, run_um, hash_gen1).
  Since then the file changed in two CSS COMMENTS only (diff checked) → now 3181f011.
- Docs: CLAUDE_v2.74.md rewritten (both lanes); CHANGELOG_v2.74.md lane B bullets + departures + closing sections
  written, numbers left as ⟦…⟧ placeholders to fill from the final runs; README table/stage plan/verify updated (⟦⟧ too).
- NEXT: fill numbers, final shots (16) into html/evidence/v2.74 + look, re-run key probes on 3181f011, report.
- 18:45 FINAL BYTES 3181f011 (frozen in scratchpad/v274b/final2/). Results on 5604c9ea (comment-only diff, checked):
  fixpass v2.74+v2.73 220/0; fixpass v2.73 70/146 (216); census 0 of 2414; mobileshell 279/0; desktopshell 154/4
  (v2.73's four); RC 8 = v2.73's FAIL lines line for line (shellui 16/3, domainrail 14/6, cogmodal 33/0,
  assetsbtn 13/1, settleparams 28/2, geninfo 21/0, worldreset 35/0, typepreview 41/0); run.sh 1280/0; run_um 882/0;
  hash_gen1 ALL IDENTICAL. 16 shots taken on 3181f011 into html/evidence/v2.74 (lane A's 13 interim captures
  deleted by shots_v274.js: the brief's <=16 bar) — ALL 16 LOOKED AT.
- 18:55 docs: CHANGELOG ⟦SHELL⟧/⟦FLAKE⟧ filled, Q10 "shown in captures" claim corrected, bytes note added; README
  mobile/desktop expectations and RC paragraph (v2.74) filled; CLAUDE_v2.74 DS-02 attribution corrected.
  Final fixpass re-run on 3181f011 with final labels running (G_fixpass_v274/v273.out) → then fill ⟦FIXPASS⟧,
  ⟦FIXPASS_DETAIL⟧, ⟦README_FIXPASS⟧, ⟦README_FIXPASS73⟧ and report.
- 19:10 G runs on 3181f011: probe_fixpass v2.74 (+v2.73 before) 220/0; v2.73 alone 70/146 of 216 (63 "as before",
  2 set-up, 5 name the v2.73 guarantee kept). Lane A's probe_v274fix.js on the final bytes (PROBE_ONLY V02/V08/V09):
  19/4 — its two V09 ink assertions x2 themes fail BY DESIGN (V24/V29 washed .pill.on and the Base map row);
  superseded by probe_fixpass.js → the main loop should git rm it.
- 19:12 one more COMMENT edit: the bake-key comment pointed at tests/perf/probe_v274fix.js → now probe_fixpass.js
  ("P7 — the bake-cache key"). Parse OK. FINAL BYTES bc0559a4ec776578672ba31c88d46ecd (diff vs 3181f011: that one
  comment line). Smoke run of P7|V26|V24 on bc0559a4 in background (out/H_fixpass_final3.out).
- 19:25 smoke on bc0559a4 (PROBE_ONLY P7|V26|V24): 75 passed, 0 failed (exit 3 = filtered, not a verdict).
  LANE B DONE — report sent via StructuredOutput.
- All ⟦⟧ placeholders filled (0 left). CLAUDE_v2.74: DS-02 attribution fixed. CHANGELOG: Q10 capture claim fixed,
  comment-only-bytes note, flaky-run count 5 (3 pass, 2 fail one each).
