// How many buildRoutingSurface() calls in one generate() rebuild from an input identical to an earlier
// call's? Wraps the global (timing + an FNV hash of the input field's bytes, sea level and the world
// flag; the call itself is unchanged) and also hashes each OUTPUT, so "same input -> same output" is
// checked, not assumed. Also counts the share of heap pushes that are interior ocean cells (sub-sea
// cells whose 8 neighbours are all sub-sea, which can close nothing when popped).
// Usage: node routing_dupes.js <file.html> <res> [seed]
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const [file, res, seed] = process.argv.slice(2);
  const b = await chromium.launch();
  const page = await b.newPage({ viewport: { width: 1400, height: 900 } }); page.on('dialog', d => d.dismiss().catch(() => {}));
  await page.goto('file://' + path.resolve(file), { waitUntil: 'load', timeout: 180000 });
  await page.waitForTimeout(800);
  const r = await page.evaluate(async ({ R, S }) => {
    const fnv = a => { const u = new Uint8Array(a.buffer, a.byteOffset, a.byteLength); let h = 2166136261 >>> 0; for (let i = 0; i < u.length; i++) { h ^= u[i]; h = Math.imul(h, 16777619) >>> 0; } return h >>> 0; };
    const calls = [];
    const o = window.buildRoutingSurface;
    window.buildRoutingSurface = function (fld, W, H, wrapX, sea) {
      const inH = fnv(fld) + ':' + sea + ':' + !!wrapX + ':' + W + 'x' + H;
      // interior-ocean share of the initial heap
      let sub = 0, interior = 0;
      for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) { const i = y * W + x; if (!(fld[i] < sea)) continue; sub++;
        let all = true; for (let dy = -1; dy <= 1 && all; dy++) for (let dx = -1; dx <= 1; dx++) { if (!dx && !dy) continue; let nx = x + dx; const ny = y + dy; if (wrapX) nx = ((nx % W) + W) % W; else if (nx < 0 || nx >= W) { all = false; break; } if (ny < 0 || ny >= H) { all = false; break; } if (!(fld[ny * W + nx] < sea)) { all = false; break; } }
        if (all) interior++; }
      const t = performance.now(); const out = o.apply(this, arguments); const ms = performance.now() - t;
      calls.push({ inH, outH: fnv(out), ms: +ms.toFixed(0), cells: W * H, subSea: sub, interiorOcean: interior, caller: (new Error().stack.split('\n')[2] || '').trim().replace(/\(.*\)/, '').slice(0, 40) });
      return out;
    };
    state.tect.seed = S; state.resW = R; _setupSkipped = true; GW = R; GH = gridH(GW); allocate();
    await generate();
    window.buildRoutingSurface = o;
    const seen = new Map(); let dup = 0, dupMs = 0, mismatch = 0;
    for (const c of calls) { if (seen.has(c.inH)) { dup++; dupMs += c.ms; if (seen.get(c.inH) !== c.outH) mismatch++; } else seen.set(c.inH, c.outH); }
    return { calls, builds: calls.length, duplicateInputBuilds: dup, duplicateMs: dupMs, totalMs: calls.reduce((s, c) => s + c.ms, 0), sameInputDifferentOutput: mismatch };
  }, { R: +res, S: +(seed || 12345) });
  console.log(JSON.stringify({ file: path.basename(file), res: +res, seed: +(seed || 12345), ...r }));
  await b.close();
})();
