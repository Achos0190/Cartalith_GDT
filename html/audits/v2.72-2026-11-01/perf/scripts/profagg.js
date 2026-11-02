// Aggregate a CDP Profiler profile by function (name:line in the HTML): self and inclusive ms.
module.exports = function aggregate(profile, n) {
  const dt = new Map(); profile.samples.forEach((s, i) => dt.set(s, (dt.get(s) || 0) + (profile.timeDeltas[i] || 0)));
  const byId = new Map(profile.nodes.map(x => [x.id, x])); const parent = new Map();
  for (const x of profile.nodes) for (const c of (x.children || [])) parent.set(c, x.id);
  const key = x => (x.callFrame.functionName || '(anon)') + ':' + (x.callFrame.lineNumber + 1);
  const self = new Map(), total = new Map();
  for (const x of profile.nodes) {
    const us = dt.get(x.id) || 0; if (!us) continue;
    self.set(key(x), (self.get(key(x)) || 0) + us);
    const seen = new Set(); let id = x.id;
    while (id != null) { const k = key(byId.get(id)); if (!seen.has(k)) { seen.add(k); total.set(k, (total.get(k) || 0) + us); } id = parent.get(id); }
  }
  const top = (m, k) => [...m.entries()].sort((a, b) => b[1] - a[1]).slice(0, k).map(([name, us]) => [name, Math.round(us / 1000)]);
  return { self: top(self, n), total: top(total, n * 2) };
};
