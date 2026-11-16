cd /data/local/tmp
rm -f fgl_out.txt
for km in 1200 40075; do
  for lv in 10 16 6 13; do
    echo "=== km=$km level=$lv $(date +%T)" >> fgl_out.txt
    TILE_BENCH_KM=$km TILE_BENCH_LEVELS=$lv TILE_BENCH_RUNS=7 ./fgl_bench deep_tile_cost_by_level --ignored --nocapture --test-threads 1 2>&1 | grep -E "start|map |level|panicked|test result" >> fgl_out.txt
    sleep 15
  done
done
echo ALLDONE >> fgl_out.txt
