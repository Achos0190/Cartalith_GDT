#!/usr/bin/env bash
# Interleaved A/B driver for the stand-alone scripts (shell.js, slider.js, split_event.js, ...):
# N samples per file, ABBA order, one fresh process per sample.
# Usage: driver2.sh <script.js> <N> <out.jsonl> [extra args...]
set -u
cd /tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-perf
export NODE_PATH=/opt/node-tools/node_modules
S=$1; N=$2; OUT=$3; shift 3
A=v271.html; B=v272.html
: > "$OUT"
for i in $(seq 1 "$N"); do
  if (( i % 2 )); then ORDER="$A $B"; else ORDER="$B $A"; fi
  for f in $ORDER; do
    echo "[$(date +%T)] $S sample $i $f (load: $(cut -d' ' -f1-3 /proc/loadavg))" >&2
    timeout 900 node "scripts/$S" "$f" "$@" >> "$OUT" || echo "{\"file\":\"$f\",\"fatal\":\"exit $?\"}" >> "$OUT"
  done
done
echo "[$(date +%T)] done $S" >&2
