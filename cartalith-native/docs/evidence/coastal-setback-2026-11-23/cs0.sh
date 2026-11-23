#!/bin/bash
# cs0.sh <dir>: the hazard census over every dump in <dir>
E=C:/cw/cs_target/release/deps/cartalith_godot-b23e1d50349921aa.exe
D=$1; mkdir -p $D/rows
for f in $D/*.jsonl; do t=$(basename $f .jsonl); P=""; [[ $t == w532187* ]] && P=C:/cw/owner_repro/seed532187_params.txt
 env ${P:+CS0_PARAMS=$P} CS0_DUMP=$f CS0_OUT=$D/rows $E cs0_hazard_census --ignored --nocapture --test-threads 1 > $D/cs0_$t.txt 2>&1; echo "cs0 $t $?" >> $D/status.txt; done
