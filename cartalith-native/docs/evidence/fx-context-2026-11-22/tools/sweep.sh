#!/bin/bash
# one pass over the three sizes: load (ROCm, gpu max, parallel 1), 5 params + 5 edit_ridge runs, unload.
T=C:/cw/fx/cartalith-native/docs/evidence/fx-context-2026-11-22/tools
for CTX in 8192 12288 16384; do
  OUT=C:/cw/fx_scratch/c$CTX; mkdir -p $OUT
  lms load qwen3.8-27b --gpu max -c $CTX --parallel 1 -y > $OUT/load.txt 2>&1
  lms ps > $OUT/ps.txt 2>&1
  bash $T/vram.sh $OUT/vram.txt & VP=$!
  for n in 1 2 3 4 5; do bash $T/runlive.sh $CTX params $n; bash $T/runlive.sh $CTX edit_ridge $n; done
  kill $VP 2>/dev/null
  lms unload --all
done
echo SWEEPDONE
