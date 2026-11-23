#!/bin/bash
# usage: run_census.sh <projdir> <outdir>
G="C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
P=$1; O=$2; mkdir -p $O
export APPDATA=C:/cw/cs_appdata LOCALAPPDATA=C:/cw/cs_appdata CARTALITH_CONFIG_PATH=C:/cw/cs_appdata/cfg.ini
cd $P
"$G" --headless --path . --script res://_towncensus_probe.gd -- --seeds 483920,912345,246371,83744 --max-quartered 6 > $O/census_384.txt 2>&1; echo "census384 $?" >> $O/status.txt
"$G" --headless --path . --script res://_towncensus_probe.gd -- --seeds 532187 --gw 2048 --gh 1311 --params C:/cw/owner_repro/seed532187_params.txt --cell 1478,1159 --cell-min-housed 90 --max-quartered 0 > $O/census_owner.txt 2>&1; echo "owner $?" >> $O/status.txt
"$G" --headless --path . --script res://_towncensus_probe.gd -- --seeds 483920 --gw 2048 --gh 1536 --max-quartered 2 > $O/census_2048.txt 2>&1; echo "census2048 $?" >> $O/status.txt
for a in "483920 2048 1311" "532187 2048 1311 P" "483920 384 288" "912345 384 288" "246371 384 288" "83744 384 288" "483920 384 288 A"; do
 set -- $a; extra=""; tag=w$1_$2
 [ "$4" = "P" ] && extra="--params C:/cw/owner_repro/seed532187_params.txt"
 [ "$4" = "A" ] && extra="--app" && tag=app$1_$2
 "$G" --headless --path . --script res://cs0_probe.gd -- --seed $1 --gw $2 --gh $3 $extra --out $O/$tag.jsonl > $O/probe_$tag.log 2>&1; echo "probe $tag $?" >> $O/status.txt
done
echo ALLDONE >> $O/status.txt
