#!/bin/bash
# the OLD flow (origin/main's catalogue.rs and params_flow.rs): three params runs at 12288, only for the token counts
T=C:/cw/fx/cartalith-native/docs/evidence/fx-context-2026-11-22/tools
lms load qwen3.8-27b --gpu max -c 12288 --parallel 1 -y > /dev/null 2>&1
for n in 11 12 13; do bash $T/runlive.sh 12288 params $n; done
lms unload --all > /dev/null 2>&1
echo BEFOREDONE
