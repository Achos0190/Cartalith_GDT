#!/usr/bin/env bash
# RK-8: one model's live knowledge evaluation, then the report over every model run so far.
#
#   tools/rk8_live.sh <preset> <model> <label> <out-dir> [more driver arguments...]
#   e.g. tools/rk8_live.sh lm_studio qwen3-14b qwen3-14b "$PWD/docs/evidence/rag-2026-11-12/runs" --repeats 3
#   <out-dir> must be an ABSOLUTE path (the driver and the report run from different directories).
#
# WHAT it does: runs `godot-project/_llmknow_eval_cli.tscn` for the three evaluation worlds, both modes
# (tools, pre) and the number of repeats given (default 3), writing <label>__<seed>.jsonl under <out-dir> (a rerun with the same label replaces its files);
# then prints the markdown report for every .jsonl in <out-dir> (`cargo run -p cartalith-knowledge
# --example rk8_report`). Run it once per model; the report accumulates.
#
# WHAT it never does: load, unload or switch a model (the owner loads the model in LM Studio first and
# sets thinking off in its profile; a hosted model is reached with --key-env VAR). It does not pass --fake.
# Set GODOT to the Godot 4.7.2 console executable if `godot` is not on the PATH.
set -euo pipefail
if [ "$#" -lt 4 ]; then
  sed -n "2,16p" "$0"
  exit 2
fi
preset="$1"; model="$2"; label="$3"; out="$4"; shift 4
godot_bin="${GODOT:-godot}"
here="$(cd "$(dirname "$0")/.." && pwd)"
(cd "$here/godot-project" && "$godot_bin" --headless --path . res://_llmknow_eval_cli.tscn -- \
  --preset "$preset" --model "$model" --label "$label" --out-dir "$out" "$@")
(cd "$here" && cargo run -q -p cartalith-knowledge --example rk8_report -- "$out" "Live evaluation ($(date +%Y-%m-%d))")
