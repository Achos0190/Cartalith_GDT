#!/bin/bash
# usage: run.sh <tag> <gw> <reps> <warm> <gpu 0|1> [keys] [mode]   (env CIV=0 to skip civ)
EV=C:/cw/bench16k/cartalith-native/docs/evidence/bench-16k-32k-2026-11-16
tag=$1; export SNAP_GW=$2 SNAP_REPS=$3 SNAP_WARM=$4 SNAP_GPU=$5 SNAP_WORLD=1 SNAP_SEED=1200 SNAP_CIV=${CIV:-1}
[ -n "$6" ] && export SNAP_GPU_KEYS=$6
[ -n "$7" ] && export SNAP_GPU_MODE=$7
powershell -NoProfile -File "$EV/sampler.ps1" -ProcName benchgen -Out "$EV/$tag.csv" &
SP=$!
sleep 1
C:/cw/bench16k_target/benchgen.exe device_stage_bench --ignored --nocapture --test-threads 1 > "$EV/$tag.log" 2>&1
wait $SP
echo finished $tag
