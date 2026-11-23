#!/bin/bash
G="C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
O=C:/cw/cs_runs/after3; mkdir -p $O
export APPDATA=C:/cw/cs_appdata3 LOCALAPPDATA=C:/cw/cs_appdata3 CARTALITH_CONFIG_PATH=C:/cw/cs_appdata3/c.ini
cd C:/cw/cs_after/godot-project
"$G" --headless --path . --script res://_towncensus_probe.gd -- --seeds 483920 --gw 2048 --gh 1536 --max-quartered 2 --rows > $O/census_2048_rows.txt 2>&1; echo "rows $?" >> $O/status.txt
"$G" --headless --path . --script res://_towncensus_probe.gd -- --seeds 532187 --gw 2048 --gh 1311 --params C:/cw/owner_repro/seed532187_params.txt --cell 1478,1159 --cell-min-housed 90 --max-quartered 0 --rows > $O/census_owner.txt 2>&1; echo "owner $?" >> $O/status.txt
"$G" --headless --path . --script res://cs0_probe.gd -- --seed 483920 --gw 2048 --gh 1311 --out $O/w483920_2048.jsonl > $O/probe4.log 2>&1; echo "probe h4 $?" >> $O/status.txt
"$G" --headless --path . --script res://cs0_probe.gd -- --seed 532187 --gw 2048 --gh 1311 --params C:/cw/owner_repro/seed532187_params.txt --out $O/w532187_2048.jsonl > $O/probe4o.log 2>&1; echo "probe owner $?" >> $O/status.txt
for h in 6 8; do mkdir -p $O/h$h; CS_H_SURGE=$h "$G" --headless --path . --script res://cs0_probe.gd -- --seed 483920 --gw 2048 --gh 1311 --out $O/h$h/w483920_2048.jsonl > $O/h$h/probe.log 2>&1; echo "probe h$h $?" >> $O/status.txt; done
echo ALLDONE >> $O/status.txt
