#!/usr/bin/env bash
# Sixth queue: after queue5's ALL DONE 5. Viewport geometry at three window sizes, both versions.
set -u
cd /tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-perf
export NODE_PATH=/opt/node-tools/node_modules
R=results
log(){ echo "[$(date +%T)] $* (load: $(cut -d' ' -f1-3 /proc/loadavg))" >> $R/queue6.log; }
until grep -q "ALL DONE 5" $R/queue5.log 2>/dev/null; do sleep 3; done
for wh in "1400 900" "1920 1080" "1100 800"; do for f in v271.html v272.html; do log "geom $f $wh"; timeout 300 node scripts/geom.js $f $wh >> $R/geom.jsonl 2>&1; done; done
log "ALL DONE 6"
