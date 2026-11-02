#!/usr/bin/env bash
# Third queue: after queue2's ALL DONE 2. Listener growth, both versions.
set -u
cd /tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-perf
export NODE_PATH=/opt/node-tools/node_modules
R=results
log(){ echo "[$(date +%T)] $* (load: $(cut -d' ' -f1-3 /proc/loadavg))" >> $R/queue3.log; }
until grep -q "ALL DONE 2" $R/queue2.log 2>/dev/null; do sleep 3; done
log "listeners v271"; timeout 600 node scripts/listeners.js v271.html > $R/listeners_v271.json 2>&1
log "listeners v272"; timeout 600 node scripts/listeners.js v272.html > $R/listeners_v272.json 2>&1
log "ALL DONE 3"
