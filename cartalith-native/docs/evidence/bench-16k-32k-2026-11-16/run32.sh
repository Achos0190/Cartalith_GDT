#!/bin/bash
# 32k run with a commit-limit watchdog: kills ONLY the benchgen process this script started
# if system free virtual memory (commit headroom) falls below 3 GB, or after MAXMIN minutes.
EV=C:/cw/bench16k/cartalith-native/docs/evidence/bench-16k-32k-2026-11-16
tag=r32k_cpu; export SNAP_GW=32768 SNAP_REPS=1 SNAP_WARM=0 SNAP_GPU=0 SNAP_WORLD=1 SNAP_SEED=1200 SNAP_CIV=${CIV:-1}
powershell -NoProfile -File "$EV/sampler.ps1" -ProcName benchgen -Out "$EV/$tag.csv" &
SP=$!
sleep 1
C:/cw/bench16k_target/benchgen.exe device_stage_bench --ignored --nocapture --test-threads 1 > "$EV/$tag.log" 2>&1 &
BP=$!
start=$(date +%s)
while kill -0 $BP 2>/dev/null; do
  sleep 5
  free=$(powershell -NoProfile -c "[int]((Get-CimInstance Win32_OperatingSystem).FreeVirtualMemory/1024)")
  echo "$(date +%s) free_virtual_mb=$free" >> "$EV/$tag.watchdog.txt"
  if [ "${free:-99999}" -lt 3000 ]; then echo "WATCHDOG: free virtual $free MB < 3000, killing benchgen" | tee -a "$EV/$tag.watchdog.txt"; taskkill //F //IM benchgen.exe; break; fi
  if [ $(( $(date +%s) - start )) -gt $(( ${MAXMIN:-240} * 60 )) ]; then echo "WATCHDOG: time cap" | tee -a "$EV/$tag.watchdog.txt"; taskkill //F //IM benchgen.exe; break; fi
done
wait $SP
echo finished $tag
