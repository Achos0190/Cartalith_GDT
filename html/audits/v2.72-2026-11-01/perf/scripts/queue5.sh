#!/usr/bin/env bash
# Fifth queue: after queue4's ALL DONE 4. Routing-surface duplicate builds (engine is identical in
# both versions, so v2.72 only), two seeds, 512 and 1024.
set -u
cd /tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-perf
export NODE_PATH=/opt/node-tools/node_modules
R=results
log(){ echo "[$(date +%T)] $* (load: $(cut -d' ' -f1-3 /proc/loadavg))" >> $R/queue5.log; }
until grep -q "ALL DONE 4" $R/queue4.log 2>/dev/null; do sleep 3; done
for s in 12345 31337; do for r in 512 1024; do log "routing_dupes $r seed $s"; timeout 600 node scripts/routing_dupes.js v272.html $r $s >> $R/routing_dupes.jsonl 2>&1; done; done
log "ALL DONE 5"
