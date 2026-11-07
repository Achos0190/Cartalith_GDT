#!/usr/bin/env bash
# Fourth queue: after queue3's ALL DONE 3. Wheel attribution, the Sun-slider probe, hash_gen1.
set -u
cd /tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-perf
export NODE_PATH=/opt/node-tools/node_modules
R=results
log(){ echo "[$(date +%T)] $* (load: $(cut -d' ' -f1-3 /proc/loadavg))" >> $R/queue4.log; }
until grep -q "ALL DONE 3" $R/queue3.log 2>/dev/null; do sleep 3; done
log "wheel_attr 1024 v272"; timeout 900 node scripts/wheel_attr.js v272.html 1024 > $R/wheelattr1024_v272.json 2>&1
log "sun_probe v271"; timeout 300 node scripts/sun_probe.js v271.html > $R/sun_v271.txt 2>&1
log "sun_probe v272"; timeout 300 node scripts/sun_probe.js v272.html > $R/sun_v272.txt 2>&1
log "hash_gen1 v271 vs v272"; (cd rc && PLAYWRIGHT_DIR=/opt/node-tools/node_modules/playwright timeout 900 node tests/perf/hash_gen1.js ../v271.html ../v272.html) > $R/hash_gen1.txt 2>&1
log "ALL DONE 4"
