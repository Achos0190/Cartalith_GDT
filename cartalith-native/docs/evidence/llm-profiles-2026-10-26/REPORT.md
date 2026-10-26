# Assistant profiles: evidence (2026-10-26, Ruling DQ)

Method: windowed Godot 4.7.2 from a scratch project copy with its own `config/name` and a wiped user dir, one probe at a time under the shared window lock; the debug DLL was rebuilt from the same tree (DLL 18:01 after the last Rust source edit at that time; the comment-only edit that followed was rebuilt again before the final test run). No mock model server was built or extended. The request-bytes check reads `LlmStream.prepare`, which builds the request without sending it; the one live check used a real LM Studio with `qwen3-14b`.

## Rust

- `cargo test -p cartalith-llm --lib profiles`: 26 tests (serialisation round trip, validation, the 16 and 40 caps, unknown fields ignored, a newer version tolerated, never a key field, built-in values as literals, accounts, https rule, startup order).
- Mutation test (Python, exact literal replace, restore in `finally`, file hash checked): **27 of 27 mutants killed**. First pass had one survivor (the remote-profile GPU rule); a test was added and the mutant then died.
- `cargo test --workspace -j 2`: see STATUS.md for the totals.

## Probes (all PASS)

| Probe | desktop 1600x1000 | tablet 1280x800 | phone 393x852 |
|---|---|---|---|
| `_llmprofiles_probe` (new) | 166 ok | 176 ok | 172 ok |
| `_llmprofiles_probe` phase 2, a second process on the same user dir | 13 ok | | |
| `_llmset_probe` | 272 | 338 | 338 |
| `_llmpanel_probe` | 93 | 103 | 103 |
| `_llmmodels_probe` | 61 | | |
| `_llmparams_probe` | 137 | | |
| `_llmplace_probe` | 130 | | |

The new probe covers: nothing written by opening; the shipped profiles as literals (a phone's Local has an empty address); a switch applies provider, address, model, thinking, ticks, reply cap, retries and key account at once, with the master switch, Commands, Edit and the provider switches unchanged and no socket attempted; consent card before the switch completes (the gate answers `consent_needed` while it is open), decline and Esc restore every value, agree records that origin only; ticks, cap and model are separate per profile; the request bytes carry the profile's model, the pinned cap and `reasoning_effort: none` for Local, and neither for External; retry limit 0 and 1; one key entry per profile account (saved under a `pf_*` account only, never `lm_studio` or `anthropic`), a duplicate has its own account and no key, no key in the profile text or settings file; the delete card with Cancel, Esc, keep-key and forget-key paths and the built-ins refused; 16 profiles, 40-character names, duplicate names, rename; a remote profile over http is switched to with its reason and refused by the gate; restore at startup (last used, then default) and across a real process restart; the chip's right-click menu (desktop and tablet); a saved world `.zip` (every entry and the raw bytes) holds none of it, with a positive control; density floors and window fit.

## Live, real model

`_llmprofiles_live_probe` against LM Studio with `qwen3-14b` (`lms load qwen3-14b --gpu max -c 8192 --parallel 1 -y`, server on 127.0.0.1:1234, unloaded, stopped and `llama-server` killed afterwards): switching to Local with the master switch on asked for consent first; the panel's reply cap was the profile's pin (48); the reply finished with visible text; the server reported `prompt 207, completion 48, total 255`, i.e. the pinned cap was honoured with thinking off. One run, a single observation.

## After the rebase onto origin/main (which brought the phone GUI pass and PW-3)

The profile probe at all three sizes, `_llmset_probe` and `_llmpanel_probe` at all three, and `_quickbar_assistant_probe`, `_llmstream_probe`, `_llmhosted_probe`, `_llmquery_probe`, `_llmknow_probe` at desktop PASS. `_quickbar_assistant_probe` asserted the chip's menu held exactly three rows; it now expects the profile rows ahead of the two it always had (a deliberate change to a regression gate). **`_llmview_probe` fails 8 checks** (a window row reading "will be opened", the shortcuts window opening, "Apply 4 view changes") **identically on origin/main's own GDScript** (run from a second scratch copy of `git archive origin/main` with the same DLL), so it is not caused by this change; it is left for whoever owns PW-3.

## Findings

- **The phone sheets' stock Close button measures 29 px tall** (the Assistant panel and Assistant settings). `phone_window` floors it to 44 px before `phone_fit` resizes the sheet, and the floor is lost; two attempts to re-seat it after the resize did not move it and were reverted. Every control this work added is 44 px or more. Not fixed here.
- Opening the consent card from inside an open panel or settings window prints Godot's "exclusive child" console error (the card and the window are both exclusive children of the app). The settings window's existing consent card has the same shape; it is cosmetic.
- The Delete card's two key answers were shortened ("Delete, forget key" / "Delete, keep key") after the longer ones ran to the edge of a 393 px phone.

## Screenshots

`llmprof_desktop_*`, `llmprof_tablet_*`, `llmprof_phone_*` in this folder: the panel header with the switch (12_panel_header), the settings PROFILE group (12_settings_profile), the chip's menu (10_chip_menu, desktop), the consent card on a switch (3_consent_card) and the delete card (6_delete_card).
