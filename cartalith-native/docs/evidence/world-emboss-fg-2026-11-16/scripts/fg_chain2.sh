#!/bin/bash
cd C:/cw
until [ -f C:/cw/fg_chain.done ]; do sleep 20; done
D="x4;x8;x32;s471;s78;s50;s25"
./fg_run.sh deep_after C:/cw/fgfix_dll/after2.dll C:/cw/fg_deep_after --pivot 0.40,0.625 --cfg "world40075,1,40075,1024,512,$D" --cfg "world2048,1,40075,2048,1024,z759;x32;s471;s78;s50;s25"
./fg_run.sh deep_before C:/cw/fgfix_dll/before.dll C:/cw/fg_deep_before --pivot 0.40,0.625 --cfg "world40075,1,40075,1024,512,$D" --cfg "world2048,1,40075,2048,1024,z759;x32;s471;s78;s50;s25"
./fg_run.sh deepm_after C:/cw/fgfix_dll/after2.dll C:/cw/fg_deepm_after --pivot 0.45,0.48 --cfg "world40075,1,40075,1024,512,$D" --cfg "region512,0,1200,512,384,z900;x32;s50"
./fg_run.sh deepm_before C:/cw/fgfix_dll/before.dll C:/cw/fg_deepm_before --pivot 0.45,0.48 --cfg "world40075,1,40075,1024,512,$D" --cfg "region512,0,1200,512,384,z900;x32;s50"
echo DONE > C:/cw/fg_chain2.done
