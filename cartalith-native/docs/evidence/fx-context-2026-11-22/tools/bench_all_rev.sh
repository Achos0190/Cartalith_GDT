#!/bin/bash
T=C:/cw/fx/cartalith-native/docs/evidence/fx-context-2026-11-22/tools
for CTX in 16384 12288 8192; do
  lms load qwen3.8-27b --gpu max -c $CTX --parallel 1 -y > /dev/null 2>&1
  bash $T/vram.sh C:/cw/fx_scratch/bench2_vram_$CTX.txt & VP=$!
  python $T/bench.py $CTX
  kill $VP 2>/dev/null
  lms unload --all > /dev/null 2>&1
done
echo BENCHDONE
