# MISTAKES_CANDIDATES_2026-10-26.md

**Candidate entries for the owner to copy into `MISTAKES.md` if they agree.** This file is a staging area, not the register: `MISTAKES.md` itself was not edited, and nothing here is a rule until the owner moves it. Written 2026-10-25/26 from the LLM-3 to LLM-6 sessions and the local-model setup session (session `01664rQgzKuqZJNTsWcNrFHs` wrote this file; the other sessions are named per entry where known).

**Evidence tags.** **[verified]** = checked in the repository by the writer of this file against a named place. **[reported by agent]** = told to the writer by the session that hit it (or by the briefing), not independently reproduced here; treat as unconfirmed until the owner or a verifier confirms. `MISTAKES.md`'s own rule is "do not record unverified guesses", so the [reported by agent] entries are the ones to hold back until confirmed.

Each entry gives a suggested preflight trigger (the left-column phrase), the root cause, the rule, how to verify, and the session. Overlap with an existing `MISTAKES.md` row is noted so the owner can merge instead of duplicating.

---

## 1. Appending to a packed array fetched from a Dictionary is lost

- **Preflight trigger:** *Mutate a `Packed*Array` you read out of a `Dictionary` or `Array`*
- **Root cause:** GDScript `Packed*Array`s have value semantics. `var a = dict["k"]` copies, `a.append(x)` changes the copy, and the dictionary still holds the old array. The code looks right and runs without error.
- **Rule:** after mutating, store it back (`dict["k"] = a`), or mutate in place through the dictionary expression only where the engine documents it as by-reference. Never rely on `append` on a fetched local.
- **How to verify:** a probe that appends, then reads the dictionary entry's `size()` and asserts it grew; mutation test: remove the re-store, the probe goes red.
- **Session:** an LLM-3 to LLM-6 build session. **[reported by agent]**: no repository place was named, so this was not re-opened at a symbol.

## 2. `netstat -p TCP` shows IPv4 only; check TCPv6 as well

- **Preflight trigger:** *Assert "zero sockets" or "nothing is listening" from `netstat`*
- **Root cause:** `netstat -p TCP` on Windows lists only IPv4. `localhost` can resolve to `::1`, so a connection to a local model server can be on TCPv6 and be invisible to the check, which then reports a clean zero it has not earned.
- **Rule:** run both `netstat -p TCP` and `netstat -p TCPv6` (or `netstat -ano` and filter) for any zero-connection claim; state both counts.
- **How to verify:** open a deliberate loopback connection to `[::1]` and confirm the check counts it; mutation: drop the TCPv6 pass, the test goes red.
- **Session:** LLM-2 to LLM-4 (the zero-socket-when-off bar). **[verified in part]**: `cartalith-native/docs/STATUS.md` (the LLM-4 and LLM-3 entries) says the `netstat` checks ran "over both address families" and show 0 non-listening rows. That confirms the practice, not the failure that caused it.

## 3. A worker thread cannot build Godot values in this gdext build, and a script method cannot be called from `NOTIFICATION_PREDELETE`

- **Preflight trigger:** *Move Godot-object work onto a `Thread`, or free something in `_notification`*
- **Root cause:** this build of gdext has no `experimental-threads`, so a `Dictionary` / `Array` / `GString` operation off the main thread panics. Separately, during `NOTIFICATION_PREDELETE` the script instance is going away, so calling a script method from it is not safe.
- **Rule:** a worker thread handles raw bytes only; the main thread drains a queue and builds every Godot value. In `NOTIFICATION_PREDELETE`, free raw RIDs and native resources only; never call back into script.
- **How to verify:** the thread half is already stated in `LLM_ASSISTANT_SCOPE.md` s.2 (the rule `landmark_run` and `region_new_world` document). The predelete half: `map_overlay.gd`'s `_notification` frees raw RIDs only, which is consistent with it.
- **Session:** LLM-3 (wire worker thread). **Thread half [verified]** at `LLM_ASSISTANT_SCOPE.md` s.2; **predelete half [reported by agent]**: the symbol was not found documenting the prohibition itself.
- **Overlap:** none in the preflight table found by keyword (`PREDELETE`, `experimental-threads` absent from `MISTAKES.md`).

## 4. A Python text-mode rewrite on Windows turns LF into CRLF; mutation harnesses must restore original bytes

- **Preflight trigger:** *Run a mutation harness that rewrites a source file, or edit a file with a script*
- **Root cause:** `open(path).read()` then `open(path, "w").write()` on Windows translates newlines. A harness that "restores" the original text this way leaves every line ending changed, a whole-file diff, and possibly a stale-binary rebuild.
- **Rule:** read and write in binary (`rb` / `wb`), or pass `newline=""`; restore the file from the exact original bytes (saved copy or `git checkout -- path`), then confirm `git diff --stat` is empty before and after.
- **How to verify:** after a harness run, `git status` shows the file clean and `git diff --stat` is empty; mutation: run the harness in text mode and see the diff appear.
- **Session:** an LLM-6 mutation-harness session. **[reported by agent]**.

## 5. `subprocess.run(timeout=)` on the Godot console launcher hangs on the child's pipe; use `Popen` plus `taskkill /T`

- **Preflight trigger:** *Drive a headless Godot (or any launcher that spawns a child) from Python with a timeout*
- **Root cause:** the Godot console launcher spawns the real process as a child that inherits the pipe. On timeout `subprocess.run` kills the launcher only; the orphaned child keeps the pipe open, so the read after the kill never returns and the harness hangs past its timeout.
- **Rule:** use `Popen`, and on timeout run `taskkill /F /T /PID <pid>` to kill the tree, then read what remains; never `communicate()` without a bounded wait after a kill.
- **How to verify:** a probe that sleeps past the timeout returns within the timeout plus a small constant, and no `godot` process remains (`tasklist`).
- **Session:** the LLM-6c stage-1/stage-2 driver work (`_llmparams_cli.gd`). **[reported by agent]**.

## 6. The stale-binary trap: a stale DLL reports a healthy pass

- **Preflight trigger:** *Declare green after a Rust change* (an existing row covers it)
- **Root cause:** `cargo test` does not rebuild `target/debug/cartalith_godot.dll`; every `.tscn` probe loads that DLL, so a probe run after a `.rs` edit and before `cargo build` tests the previous engine and passes.
- **Rule:** compare the DLL's mtime with the newest touched `.rs` file and state both.
- **How to verify:** `ls -l --time-style=full-iso target/debug/cartalith_godot.dll` against the newest `.rs`; mutation: touch a `.rs`, rerun the probe, and the comparison must flag it.
- **Session:** recurring (LLM-5 and LLM-6 among them). **[verified]**: already `MISTAKES.md` preflight row "Grade a Godot probe as evidence for a Rust change" (the Godot half) and the stale-binary working rule in `CLAUDE.md`. **Merge into that row by raising its `×N` and adding the instance; do not add a new entry.**

## 7. Two agents running windowed Godot at once steal each other's mouse input; use the `C:/cw/.win_lock` convention

- **Preflight trigger:** *Run a windowed (non-headless) Godot probe, screenshot or click test while another agent may be doing the same*
- **Root cause:** synthetic and real mouse input are delivered to whichever window has the foreground. Two windowed runs on one desktop interleave, so clicks land in the wrong app and both runs fail or, worse, one passes on the other's click.
- **Rule:** take the lock before a windowed run (`C:/cw/.win_lock`, created exclusively, removed in a `finally`), and wait or run headless if it is held; headless runs need no lock.
- **How to verify:** two concurrent windowed probes: the second must wait or refuse; mutation: skip the lock and observe the interference.
- **Session:** the concurrent-lane sessions of 2026-10-25/26. **[reported by agent]**: `win_lock` is not in any document in the repository (searched `*.md`), so the convention exists only in agent briefs; if the owner adopts the entry, the convention itself should be written down where agents will read it.

## 8. Stopping Ollama by process name `ollama*` leaves orphaned `llama-server` runners holding VRAM

- **Preflight trigger:** *Stop a local model server to free the GPU, or measure speed/VRAM after stopping one*
- **Root cause:** Ollama starts model runners as separate `llama-server` processes. Stopping `ollama*` by name kills the server and the tray app but can leave a runner alive, which keeps its model resident in VRAM. The next measurement then runs with the card partly full: this is the same condition the 4 to 7 tok/s Ollama figure in `LLM_ASSISTANT_SCOPE.md` s.9a was taken under.
- **Rule:** stop `llama-server` too, then check the GPU memory counters (dedicated usage near the compositor's ~0.3 GB) before measuring or starting another model.
- **How to verify:** read dedicated GPU memory before and after the stop (Task Manager or the performance counter) and `tasklist | findstr llama-server`; mutation: stop only `ollama*` and see the counter stay high.
- **Session:** the local-model tooling session of 2026-10-25. **[reported by agent]** for the orphaned-runner mechanism; the stale-runner condition is [verified] only as a statement in the owner-session brief and now recorded in `LLM_ASSISTANT_SCOPE.md` s.9a.

## 9. `ollama list` and similar commands auto-start the server

- **Preflight trigger:** *Check whether Ollama is running, or leave it stopped, using the `ollama` CLI*
- **Root cause:** the CLI is a client that starts the server if none answers. An "is it stopped?" check therefore starts it (and with it the tray app), and a later `ollama list` or `ollama ps` silently undoes a stop.
- **Rule:** check state without the CLI (`tasklist`, or a TCP probe of `localhost:11434` that does not connect-and-keep); run `ollama` commands only when a running server is acceptable.
- **How to verify:** stop the server, run `ollama list`, and see a process reappear; mutation: replace the check with a port probe and the process stays absent.
- **Session:** the local-model tooling session of 2026-10-25. **[reported by agent]**.

## 10. `winget` can hang behind another stuck `winget`; run the signed installer directly after hash verification

- **Preflight trigger:** *Install a tool with `winget` in a session where another install may be running*
- **Root cause:** `winget` takes a machine-wide lock; one stuck invocation (a prompt nobody sees, a download that stalls) makes every later `winget` wait forever with no output.
- **Rule:** do not queue behind it. Download the vendor's installer, verify its hash against the published value and its signature (`Get-AuthenticodeSignature`), then run it directly; say in the record which hash and which signer you checked.
- **How to verify:** `Get-AuthenticodeSignature` reports `Valid` with the expected publisher, and the hash matches the vendor's page; the install result is confirmed by launching the tool.
- **Session:** the local-model tooling session of 2026-10-25 (LM Studio 0.4.26, `LLM_ASSISTANT_SCOPE.md` s.9a). **[reported by agent]** for the hang; the install itself is recorded in s.9a.

---

## 11. A mocked test using the literal 127.0.0.1 hid a localhost -> ::1 resolution failure

The mock server listened on `127.0.0.1` and the test dialled the same literal, so the test passed while the shipped
default URL `localhost` resolved to `::1` and failed. Rule: a network test must dial the name the product ships with
and the mock must listen on both families. Verify: run the test with the product's default URL.

## 12. A fixture that cannot fail (canary words in the user's own question)

The injection canary words were placed in the user's own question, so any answer that echoed the question passed. Rule:
a fixture must be able to fail; put the canary where only the thing under test can have read it, and run it once against
a deliberately broken implementation.

## 13. A phone sheet keeps a stale child size

A phone sheet reused a child whose minimum size was set for a previous layout; the sheet kept the old size after the
content changed. Rule: re-measure children when a sheet's content changes, and check the phone leg after any relayout.

## 14. `git checkout <file>` on a file with uncommitted work loses it

An agent restored a file to clear a diff and discarded uncommitted work. Rule: never `git checkout`/`restore` a path that
has uncommitted changes you did not make; copy it aside or `git stash` first, and name the explicit path in commits.

## 15. A probe with a parse error hangs while holding the lock

A GDScript probe with a parse error never exits, and the `C:/cw/.win_lock` directory stayed held. Rule: run
`--check-only` (or parse-check) before the windowed run, always use a hard timeout, and release the lock in a trap.

## 16. Two agents installing on the phone at once collide

Same family as entry 7: two agents driving adb or installing the APK concurrently corrupt each other's state. Rule: take
the same lock convention for the phone, one device owner at a time.

## 17. An agent stopping the shared LM Studio server

An agent stopped the LM Studio server the owner leaves running on the LAN for other agents and devices. Rule: agents never
start or stop LM Studio or Ollama unless the brief says so; the server is the owner's.

## 18. A preset command-line scene path loops the splash

Putting a scene path on the export preset's command line made the app restart into the splash repeatedly. Rule: never a
scene path in the preset command line; the main scene belongs in `project.godot`.

## 19. Project path not reset on New world overwrote a world

New world inherited the previous world's project path, so the first Save overwrote that world (fixed in `5585284e`).
Rule: any "new" action resets every identity field (path, name, dirty flag), tested by Save-after-New.

## Suggested handling

- Entry 6 merges into an existing row; the rest are new triggers.
- Entries 1, 4, 5, 7, 8, 9 and 10 carry **[reported by agent]**: confirm each (one reproduction apiece) before adding, or add them with that tag visible.
- Entries 2 and 3 are partly backed by repository text, noted above.
