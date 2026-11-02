#!/usr/bin/env bash
# Everything after the shell batch, strictly one at a time (never two measurements at once).
set -u
cd /tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-perf
export NODE_PATH=/opt/node-tools/node_modules
R=results
log(){ echo "[$(date +%T)] $* (load: $(cut -d' ' -f1-3 /proc/loadavg))" >> $R/queue.log; }
until grep -q "done shell.js" $R/shell.log 2>/dev/null; do sleep 2; done
log "load re-run (independent process, 7 each)"
bash scripts/driver.sh load 7 $R/load2.jsonl > $R/load2.log 2>&1
log "trace_idle v271"; timeout 300 node scripts/trace_idle.js v271.html 512 > $R/idle_v271.json 2>&1
log "trace_idle v272"; timeout 300 node scripts/trace_idle.js v272.html 512 > $R/idle_v272.json 2>&1
log "slider 1024 (3 each)"; bash scripts/driver2.sh slider.js 3 $R/slider1024.jsonl 1024 > $R/slider1024.log 2>&1
log "split_event 1024 v272"; timeout 600 node scripts/split_event.js v272.html 1024 > $R/split1024_v272.json 2>&1
log "split_event 1024 v271"; timeout 600 node scripts/split_event.js v271.html 1024 > $R/split1024_v271.json 2>&1
log "prof_gen 2048 v272"; timeout 900 node scripts/prof_gen.js v272.html 2048 > $R/profgen2048_v272.json 2>&1
log "prof_lodwheel 1024 v272"; timeout 900 node scripts/prof_lodwheel.js v272.html 1024 1 > $R/proflod1024_v272.json 2>&1
log "gen batch (5 each, 512c,512,1024,2048)"; bash scripts/driver.sh gen 5 $R/gen.jsonl > $R/gen.log 2>&1
log "lod wheel A/B (3 each)"; bash scripts/driver2.sh prof_lodwheel.js 3 $R/lodab.jsonl 1024 1 > $R/lodab.log 2>&1
log "ALL DONE"
