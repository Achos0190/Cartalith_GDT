#!/usr/bin/env bash
# Interleaved A/B driver: N samples per file, ABBA order, one fresh process (and browser) per sample.
# Usage: driver.sh <mode> <N> <out.jsonl> [res]     (env passes through: GEN_RES, LOD_WHEEL)
set -u
cd /tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-perf
export NODE_PATH=/opt/node-tools/node_modules
MODE=$1; N=$2; OUT=$3; RES=${4:-1024}
A=v271.html; B=v272.html
: > "$OUT"
for i in $(seq 1 "$N"); do
  if (( i % 2 )); then ORDER="$A $B"; else ORDER="$B $A"; fi
  for f in $ORDER; do
    echo "[$(date +%T)] sample $i $f (load: $(cut -d' ' -f1-3 /proc/loadavg))" >&2
    timeout 900 node scripts/bench.js "$MODE" "$f" "$RES" >> "$OUT" || echo "{\"mode\":\"$MODE\",\"file\":\"$f\",\"fatal\":\"exit $?\"}" >> "$OUT"
  done
done
echo "[$(date +%T)] done $MODE" >&2
