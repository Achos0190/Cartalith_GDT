#!/usr/bin/env bash
# Second queue: starts only after queue.sh logs ALL DONE, so measurements never overlap.
set -u
cd /tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-perf
export NODE_PATH=/opt/node-tools/node_modules
R=results
log(){ echo "[$(date +%T)] $* (load: $(cut -d' ' -f1-3 /proc/loadavg))" >> $R/queue2.log; }
until grep -q "ALL DONE" $R/queue.log 2>/dev/null; do sleep 3; done
log "profile_load2 A/B (5 each)"; bash scripts/driver2.sh profile_load2.js 5 $R/profload.jsonl > $R/profload.log 2>&1
log "prof_domain 1024 v271"; timeout 600 node scripts/prof_domain.js v271.html 1024 > $R/profdomain_v271.json 2>&1
log "prof_domain 1024 v272"; timeout 600 node scripts/prof_domain.js v272.html 1024 > $R/profdomain_v272.json 2>&1
log "split_event 2048 v272"; timeout 900 node scripts/split_event.js v272.html 2048 > $R/split2048_v272.json 2>&1
log "slider 2048 v272"; timeout 900 node scripts/slider.js v272.html 2048 > $R/slider2048_v272.json 2>&1
log "ALL DONE 2"
