#!/bin/bash
cd C:/cw
S="fit;x8;x32;s50"
CF=(--cfg "w1200,0,1200,1024,512,$S" --cfg "w5000,0,5000,1024,512,$S" --cfg "w15000,0,15000,1024,512,$S" --cfg "w200,0,200,1024,512,fit;x8;x32" --cfg "w40075,0,40075,1024,512,$S" --cfg "world40075,1,40075,1024,512,fit;x4;x8;x32;s50;s471" --cfg "world2048,1,40075,2048,1024,fit;z250;z759;x8;x32;s50" --cfg "region512,0,1200,512,384,fit;z117;z900;x8;x32;s50")
./fgleft_run.sh cap_after2 C:/cw/fgleft_dll/after2.dll C:/cw/fgleft_cap_after2 --pivot 0.40,0.625 "${CF[@]}"
python fgleft/cartalith-native/docs/evidence/world-emboss-fg-2026-11-16/scripts/fg_grainoff.py C:/cw/fgleft/cartalith-native C:/cw/fgleft_target3 C:/cw/fgleft_dll/after2_grainoff.dll > fgleft_go.log 2>&1
./fgleft_run.sh b8on2 C:/cw/fgleft_dll/after2.dll C:/cw/fgleft_b8_on2 --laws 2 --pivot 0.40,0.625 --cfg "world2048,1,40075,2048,1024,fit;z250;z759;x8;x32"
./fgleft_run.sh b8off2 C:/cw/fgleft_dll/after2_grainoff.dll C:/cw/fgleft_b8_off2 --laws 2 --pivot 0.40,0.625 --cfg "world2048,1,40075,2048,1024,fit;z250;z759;x8;x32"
echo DONE > C:/cw/fgleft_chain2.done
