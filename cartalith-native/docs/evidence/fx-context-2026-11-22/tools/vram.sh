#!/bin/bash
# usage: vram.sh <out.txt>  -- appends "epoch dedicatedGB sharedGB" (largest adapter) every ~2 s until killed
while true; do
  v=$(powershell -NoProfile -c "\$d=(Get-Counter '\GPU Adapter Memory(*)\Dedicated Usage').CounterSamples|sort CookedValue -desc|select -first 1; \$s=(Get-Counter '\GPU Adapter Memory(*)\Shared Usage').CounterSamples|sort CookedValue -desc|select -first 1; '{0:N2} {1:N2}' -f (\$d.CookedValue/1GB),(\$s.CookedValue/1GB)")
  echo "$(date +%s) $v" >> "$1"
done
