#!/bin/bash
cd C:/cw
S="fit;x8;x32;s50"
./fg_run.sh after2 C:/cw/fgfix_dll/after2.dll C:/cw/fg_after2 --pivot 0.40,0.625 \
 --cfg "w200,0,200,1024,512,$S" --cfg "w1200,0,1200,1024,512,$S" --cfg "w5000,0,5000,1024,512,$S" --cfg "w15000,0,15000,1024,512,$S" --cfg "w40075,0,40075,1024,512,$S" \
 --cfg "world40075,1,40075,1024,512,fit;x4;x8;x32;s50;s471;s78;s1962;s25" \
 --cfg "world2048,1,40075,2048,1024,fit;z250;z759;x8;x32;s50" --cfg "region512,0,1200,512,384,fit;z117;z900;x8;x32;s50"
./fg_run.sh after2p1 C:/cw/fgfix_dll/after2.dll C:/cw/fg_after2_p1 --pivot 0.45,0.48 \
 --cfg "world2048,1,40075,2048,1024,fit;z250;z759;x8;x32;s50" --cfg "region512,0,1200,512,384,z117;z900;x8;x32" --cfg "world40075,1,40075,1024,512,x8;x32;s471;s78;s50;s25" --cfg "w1200,0,1200,1024,512,fit;x8;x32" --cfg "w200,0,200,1024,512,fit;x8;x32"
./fg_run.sh grain_on C:/cw/fgfix_dll/after2.dll C:/cw/fg_grain_after_on --laws 2 --pivot 0.40,0.625 --cfg "world2048,1,40075,2048,1024,fit;z250;z759;x32"
./fg_run.sh grain_off C:/cw/fgfix_dll/after_grainoff.dll C:/cw/fg_grain_after_off --laws 2 --pivot 0.40,0.625 --cfg "world2048,1,40075,2048,1024,fit;z250;z759;x32"
python fg_grainoff.py C:/cw/fgfix_base/cartalith-native C:/cw/fgfix_base_target C:/cw/fgfix_dll/before_grainoff.dll
./fg_run.sh bgrain_on C:/cw/fgfix_dll/before.dll C:/cw/fg_grain_before_on --laws 2 --pivot 0.40,0.625 --cfg "world2048,1,40075,2048,1024,fit;z250;z759;x32"
./fg_run.sh bgrain_off C:/cw/fgfix_dll/before_grainoff.dll C:/cw/fg_grain_before_off --laws 2 --pivot 0.40,0.625 --cfg "world2048,1,40075,2048,1024,fit;z250;z759;x32"
echo CHAIN-DONE > C:/cw/fg_chain.done
