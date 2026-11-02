#!/usr/bin/env bash
# Seventh queue: after queue6's ALL DONE 6. Controlled LOD wheel A/B, 4 each, ABBA.
set -u
cd /tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-perf
export NODE_PATH=/opt/node-tools/node_modules
R=results
log(){ echo "[$(date +%T)] $* (load: $(cut -d' ' -f1-3 /proc/loadavg))" >> $R/queue7.log; }
until grep -q "ALL DONE 6" $R/queue6.log 2>/dev/null; do sleep 3; done
log "lod_ctrl A/B 4 each"; bash scripts/driver2.sh lod_ctrl.js 4 $R/lodctrl.jsonl 1024 > $R/lodctrl.log 2>&1
log "ALL DONE 7"
