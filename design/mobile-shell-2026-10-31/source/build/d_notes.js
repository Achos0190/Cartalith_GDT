/* ======================================================================================================
   The vault's notes: a Markdown editor and its links, as close to a regular Markdown editor as the shell
   allows. The syntax is Obsidian's (Ruling BF): **bold**, *italic*, ~~strike~~, ==highlight==, <u>underline</u>,
   [[note]], [[note|shown text]], [[note#heading]], ![[embed]], #tags, - [ ] tasks, > [!note] callouts, tables,
   code. Notes live as plain .md files; a note is bound to a world entity by the vault's link record
   (`KnowledgeLink`), not by anything written into the file, so the owner's templates stay front-matter free.

   A link resolves to a note by name first, then to a world entity by name (a settlement, faction, landmark or
   region with no note yet), else it is unresolved. Following an entity link with no note offers to create
   the note from the matching template; following an unresolved link offers to create it.
   ====================================================================================================== */
const NOTE_KINDS = { settlement: ['Settlement', IC.settle], province: ['Province', IC.region], continent: ['Continent', IC.region], faction: ['Faction', IC.faction], culture: ['Culture', IC.faction], landmark: ['Landmark', IC.landmark], note: ['Note', IC.notes] };

function esc(s) { return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;'); }

/* ---------------- the sample vault, written from the generated world ---------------- */
function seedNotes(c) {
  const P = c.places || [], fac = (i) => FACTIONS[i] ? FACTIONS[i].name : 'Orentamwash';
  const byName = (n) => P.find((p) => p.name === n);
  const cap = (f) => { const p = P.find((q) => q.capital && q.fac === f); return p ? p.name : (P[0] ? P[0].name : 'Vhal Serai'); };
  const vs = byName('Vhal Serai') || P[0] || { name: 'Vhal Serai', tier: 'city', fac: 0, pop: 21000 };
  const kf = byName('Kess Ford') || P[1] || { name: 'Kess Ford', tier: 'town', fac: 0 };
  const gv = byName('Grey Vale') || P[4] || { name: 'Grey Vale', tier: 'town', fac: 1 };
  const lm = (c.landmarks || []); const peak = (lm.find((l) => l.kind === 'Peak') || { name: 'Mount Asker' }).name;
  const shrine = (lm.find((l) => l.kind === 'Shrine') || { name: 'Shrine of Thornwood' }).name;
  const ford = (lm.find((l) => l.kind === 'Ford') || { name: 'Ford on the Kess Ford road' }).name;
  const ent = (kind, o) => (o && o.id ? { kind, id: o.id, name: o.name } : null);
  const N = (name, kind, entity, text, extra) => Object.assign({ id: 'n' + name.toLowerCase().replace(/[^a-z0-9]+/g, '-'), name, kind, entity, text, edited: Date.now() - Math.round(Math.random() * 5) * 86400000, private: false }, extra || {});
  return [
    N(vs.name, 'settlement', ent('settlement', vs),
      '## Settlement Profile: ' + vs.name + '\n\n**Type:** ' + (vs.tier || 'town').charAt(0).toUpperCase() + (vs.tier || 'town').slice(1) + '  \n**Location:** west coast, where the [[' + kf.name + ']] road meets the sea\n\n### General Info\n\n- **Size / Population:** about ' + (vs.pop || 21000).toLocaleString('en-GB') + '\n- **Ruling authority:** [[' + fac(vs.fac) + ']]\n\n### Current Status\n\nActive. Salt and grain leave by ship; ==the harbour silts a little more each spring==.\n\n### Culture & Identity\n\nMost households keep [[The Old Rite]]; the lighthouse is its oldest shrine. #coast #trade\n\n> [!note] Open question\n> Who holds the harbour charter since the last council fell?\n\n### Known Issues / Threats\n\n- [ ] Map the harbour districts\n- [x] Name the lighthouse keeper\n- [ ] Decide what happened at [[' + kf.name + ' crossing]]\n\n### History\n\n```chronos\n* [-112] Founded on the old ford | salt traders from the north\n@ [211~260] #blue {Building} The harbour wall\n- [380] Council of the Ford | the first elected council\n- [401] Raid at ' + kf.name + ' | see the crossing note\n= [412] Today\n```\n'),
    N(fac(0), 'faction', { kind: 'faction', id: 'f0', name: fac(0) },
      '# ' + fac(0) + '\n\n**Government:** Monarchy  \n**Faith:** [[The Old Rite]]  \n**Seat:** [[' + cap(0) + ']]\n\n## Relations\n\n| With | Stance | Since |\n|---|---|---|\n| [[' + fac(1) + ']] | trade, wary | year 380 |\n| [[' + fac(3) + ']] | shared faith | always |\n| [[' + fac(2) + ']] | border raids | year 401 |\n\n## Notes\n\nThe crown taxes the salt road at [[' + kf.name + ']]. #politics\n'),
    N('The Old Rite', 'culture', { kind: 'culture', id: 'c0', name: 'The Old Rite' },
      '# The Old Rite\n\nThe faith of [[' + fac(0) + ']] and [[' + fac(3) + ']]. Pilgrims walk from [[' + vs.name + ']] to [[' + shrine + ']] each midwinter.\n\n## Practices\n\n1. Salt on the threshold at the year\'s turn\n2. No fire lit on the longest night\n3. Vows spoken at running water, often at [[' + ford + ']]\n\n> [!quote] From a pilgrim\'s song\n> *Salt for the door, water for the word.*\n\nSee also [[' + peak + '#Cultural Significance|the mountain\'s place in the rite]]. #faith #lore\n'),
    N(kf.name + ' crossing', 'note', null,
      '# ' + kf.name + ' crossing\n\nYear 401. A [[' + fac(2) + ']] raiding party crossed at [[' + ford + ']] and burned the toll house.\n\n- Who paid for the new bridge?\n- Does [[' + kf.name + ']] still owe the crown?\n\n![[' + kf.name + ' toll ledger]]\n\n#event #year-401\n'),
    N(peak, 'landmark', ent('landmark', lm.find((l) => l.name === peak)),
      '# Landmark: ' + peak + '\n\n## Type\n- Natural wonder\n\n## Description\n- Physical appearance: a bare grey summit above the [[' + gv.name + ']] valley\n- Unique features: snow until midsummer\n\n## Cultural Significance\n- Associated myths or legends: the first salt was carried down from here, by [[The Old Rite]]\'s telling\n\n## History\n- When it was built/discovered: always known\n'),
    N(gv.name, 'settlement', ent('settlement', gv),
      '## Settlement Profile: ' + gv.name + '\n\n**Type:** Town  \n**Location:** under [[' + peak + ']]\n\n### Economic Activity\n\nWool, and the mill at [[' + gv.name + ' Mill]].\n'),
    N(fac(1), 'faction', { kind: 'faction', id: 'f1', name: fac(1) },
      '# ' + fac(1) + '\n\n**Government:** Merchant republic  \n**Faith:** Lantern faith\n\nTheir factors sit in [[' + vs.name + ']] and in Port Amre. #politics #trade\n'),
    N('Campaign log', 'note', null,
      '# Campaign log\n\n## Session 12\n\n- [x] The party reached [[' + vs.name + ']]\n- [ ] Meet the harbour council\n- [ ] Follow the ledger from [[' + kf.name + ' crossing]]\n\n## Hooks\n\n- The lighthouse keeper knows who burned the toll house\n', { private: true })
  ];
}
function notesOf(c) { if (!c.state.notes) { const seeded = seedNotes(c); c.state.notes = seeded; } return c.state.notes; }
function noteByName(c, name) { const k = String(name || '').trim().toLowerCase(); return notesOf(c).find((n) => n.name.toLowerCase() === k) || null; }
function noteById(c, id) { return notesOf(c).find((n) => n.id === id) || null; }

/* World entities a link can name: settlements, factions, landmarks. Regions share their faction's name. */
function linkEntities(c) {
  const out = [];
  (c.places || []).forEach((p) => out.push({ kind: 'settlement', id: p.id, name: p.name, sub: p.tier + (FACTIONS[p.fac] ? ' · ' + FACTIONS[p.fac].name : '') }));
  FACTIONS.forEach((f, i) => out.push({ kind: 'faction', id: 'f' + i, name: f.name, sub: f.gov + ' · ' + f.rel }));
  (c.landmarks || []).forEach((l) => out.push({ kind: 'landmark', id: l.id, name: l.name, sub: l.kind + ' · ' + l.fam }));
  const faiths = []; FACTIONS.forEach((f) => { if (!faiths.includes(f.rel)) faiths.push(f.rel); }); faiths.forEach((r, i) => out.push({ kind: 'culture', id: 'c' + i, name: r, sub: 'culture · faith of ' + FACTIONS.filter((f) => f.rel === r).map((f) => f.name).join(', ') }));
  return out;
}
function entityByName(c, name) { const k = String(name || '').trim().toLowerCase(); return linkEntities(c).find((e) => e.name.toLowerCase() === k) || null; }

/* A link target's resolution: the note first, then a world entity with no note, else unresolved. */
function resolveLink(c, name) {
  const n = noteByName(c, name); if (n) return { kind: 'note', note: n, entity: n.entity || entityByName(c, n.name) };
  const e = entityByName(c, name); if (e) return { kind: 'entity', entity: e };
  return { kind: 'none', name: String(name || '').trim() };
}

/* ---------------- reading: Markdown to HTML (escaped; links carry data attributes) ---------------- */
function mdInline(src, c) {
  const stash = []; const keep = (h) => '\u0001' + (stash.push(h) - 1) + '\u0002';
  let s = String(src);
  s = s.replace(/`([^`]+)`/g, (m, a) => keep('<code>' + esc(a) + '</code>'));
  s = s.replace(/!\[\[([^\]]+)\]\]/g, (m, a) => { const r = resolveLink(c, a.split('|')[0].split('#')[0]); return keep('<span class="md-embed ' + (r.kind === 'none' ? 'md-unres' : '') + '" data-link="' + esc(a.split('|')[0].split('#')[0].trim()) + '">▣ embedded: ' + esc(a) + '</span>'); });
  s = s.replace(/\[\[([^\]|#]+)(#[^\]|]+)?(\|[^\]]+)?\]\]/g, (m, n, h, al) => {
    const name = n.trim(), r = resolveLink(c, name);
    const shown = al ? al.slice(1) : name + (h ? ' > ' + h.slice(1) : '');
    const cls = r.kind === 'note' ? 'md-link' : (r.kind === 'entity' ? 'md-link md-ent' : 'md-link md-unres');
    return keep('<a class="' + cls + '" data-link="' + esc(name) + '"' + (h ? ' data-head="' + esc(h.slice(1)) + '"' : '') + '>' + esc(shown) + '</a>');
  });
  s = s.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (m, t, u) => keep('<a class="md-ext" data-ext="' + esc(u) + '">' + esc(t) + ' ↗</a>'));
  s = s.replace(/<u>(.*?)<\/u>/g, (m, a) => keep('<u>' + esc(a) + '</u>'));
  s = esc(s);
  s = s.replace(/\*\*([^*]+)\*\*/g, '<b>$1</b>').replace(/(^|[^*\w])\*([^*\s][^*]*?)\*(?![*\w])/g, '$1<i>$2</i>').replace(/(^|[^\w])_([^_\s][^_]*?)_(?![\w])/g, '$1<i>$2</i>');
  s = s.replace(/~~([^~]+)~~/g, '<s>$1</s>').replace(/==([^=]+)==/g, '<mark>$1</mark>');
  s = s.replace(/(^|\s)#([A-Za-z][\w/-]*)/g, '$1<span class="md-tag" data-tag="$2">#$2</span>');
  return s.replace(/\u0001(\d+)\u0002/g, (m, i) => stash[+i]);
}
function mdRender(text, c) {
  const L = String(text).split('\n'); const out = []; const heads = []; let i = 0;
  let para = [];
  const flush = () => { if (para.length) { out.push('<p>' + para.map((x) => mdInline(x, c)).join('<br>') + '</p>'); para = []; } };
  if (L[0] === '---') { const end = L.indexOf('---', 1); if (end > 0) { out.push('<div class="md-fm">' + L.slice(1, end).map((x) => esc(x)).join('<br>') + '</div>'); i = end + 1; } }
  for (; i < L.length; i++) {
    const ln = L[i];
    if (/^```/.test(ln)) { flush(); const lang = ln.slice(3).trim().toLowerCase(); const buf = []; i++; while (i < L.length && !/^```/.test(L[i])) buf.push(L[i++]); out.push(lang === 'chronos' ? chronosHtml(buf, c) : '<pre><code>' + esc(buf.join('\n')) + '</code></pre>'); continue; }
    const h = /^(#{1,6})\s+(.*)$/.exec(ln);
    if (h) { flush(); const id = 'h' + heads.length; heads.push({ level: h[1].length, text: h[2].replace(/[*=~_`]/g, ''), line: i, id }); out.push('<h' + h[1].length + ' id="' + id + '" data-line="' + i + '">' + mdInline(h[2], c) + '</h' + h[1].length + '>'); continue; }
    if (/^\s*(-{3,}|\*{3,})\s*$/.test(ln)) { flush(); out.push('<hr>'); continue; }
    if (/^>\s?/.test(ln)) {
      flush(); const buf = []; while (i < L.length && /^>\s?/.test(L[i])) buf.push(L[i++].replace(/^>\s?/, '')); i--;
      const co = /^\[!(\w+)\]\s*(.*)$/.exec(buf[0] || '');
      if (co) out.push('<div class="md-callout"><div class="md-co-t">' + esc(co[1].toUpperCase()) + (co[2] ? ' · ' + mdInline(co[2], c) : '') + '</div>' + buf.slice(1).map((x) => mdInline(x, c)).join('<br>') + '</div>');
      else out.push('<blockquote>' + buf.map((x) => mdInline(x, c)).join('<br>') + '</blockquote>');
      continue;
    }
    if (/\|/.test(ln) && i + 1 < L.length && /^\s*\|?\s*:?-{3,}/.test(L[i + 1])) {
      flush(); const row = (r) => r.replace(/^\s*\|/, '').replace(/\|\s*$/, '').split('|').map((x) => x.trim());
      const head = row(ln); i += 2; const body = []; while (i < L.length && /\|/.test(L[i])) body.push(row(L[i++])); i--;
      out.push('<div class="md-tablewrap"><table><thead><tr>' + head.map((x) => '<th>' + mdInline(x, c) + '</th>').join('') + '</tr></thead><tbody>' + body.map((r) => '<tr>' + r.map((x) => '<td>' + mdInline(x, c) + '</td>').join('') + '</tr>').join('') + '</tbody></table></div>');
      continue;
    }
    const li = /^(\s*)([-*+]|\d+\.)\s+(.*)$/.exec(ln);
    if (li) {
      flush(); const ordered = /\d/.test(li[2]); const items = [];
      while (i < L.length) { const m = /^(\s*)([-*+]|\d+\.)\s+(.*)$/.exec(L[i]); if (!m) break; items.push({ ind: Math.floor(m[1].length / 2), body: m[3], line: i }); i++; } i--;
      out.push((ordered ? '<ol>' : '<ul>') + items.map((it) => { const t = /^\[([ xX])\]\s+(.*)$/.exec(it.body); const pad = it.ind ? ' style="margin-left:' + (it.ind * 18) + 'px"' : ''; return t ? '<li class="md-task"' + pad + '><span class="md-box' + (t[1] !== ' ' ? ' on' : '') + '" data-task="' + it.line + '" role="checkbox" aria-checked="' + (t[1] !== ' ') + '">' + (t[1] !== ' ' ? '✓' : '') + '</span><span class="' + (t[1] !== ' ' ? 'md-done' : '') + '">' + mdInline(t[2], c) + '</span></li>' : '<li' + pad + '>' + mdInline(it.body, c) + '</li>'; }).join('') + (ordered ? '</ol>' : '</ul>'));
      continue;
    }
    if (!ln.trim()) { flush(); continue; }
    para.push(ln.replace(/\s{2,}$/, ''));
  }
  flush();
  return { html: out.join(''), heads };
}

/* ---------------- writing: the source, syntax-coloured behind a transparent textarea ----------------
   Same characters, same monospace metrics: only spans are added, so the caret lines up with the colours. */
function mdHighlight(text) {
  let inFence = false;
  return String(text).split('\n').map((ln) => {
    if (/^```/.test(ln)) { inFence = !inFence; return '<span class="hl-code">' + esc(ln) + '</span>'; }
    if (inFence) return '<span class="hl-code">' + esc(ln) + '</span>';
    if (/^#{1,6}\s/.test(ln)) return '<span class="hl-h">' + esc(ln) + '</span>';
    let s = esc(ln);
    s = s.replace(/^(\s*)([-*+]|\d+\.)(\s)(\[[ xX]\])?/, (m, a, b, sp, t) => a + '<span class="hl-mark">' + b + '</span>' + sp + (t ? '<span class="hl-mark">' + t + '</span>' : ''));
    s = s.replace(/^(&gt;\s?)(\[![a-z]+\])?/i, (m, a, b) => '<span class="hl-mark">' + a + '</span>' + (b ? '<span class="hl-mark">' + b + '</span>' : ''));
    s = s.replace(/(!?\[\[)([^\]]*)(\]\])/g, '<span class="hl-mark">$1</span><span class="hl-link">$2</span><span class="hl-mark">$3</span>');
    s = s.replace(/(\*\*|~~|==)(?=\S)([^*~=]+?)(\1)/g, '<span class="hl-mark">$1</span><b>$2</b><span class="hl-mark">$3</span>');
    s = s.replace(/(^|\s)(#[A-Za-z][\w/-]*)/g, '$1<span class="hl-tag">$2</span>');
    s = s.replace(/(`[^`]+`)/g, '<span class="hl-code">$1</span>');
    return s;
  }).join('\n') + '\n';
}

/* ---------------- [[ autocomplete ---------------- */
function linkQueryAt(text, caret) {
  const line0 = text.lastIndexOf('\n', caret - 1) + 1; const before = text.slice(line0, caret);
  const open = before.lastIndexOf('[['); if (open < 0) return null;
  const q = before.slice(open + 2); if (/\]\]|\|/.test(q)) return null;
  const hash = q.indexOf('#');
  return { from: line0 + open, to: caret, query: hash >= 0 ? q.slice(0, hash) : q, heading: hash >= 0 ? q.slice(hash + 1) : null };
}
function fuzzyScore(name, q) {
  const n = name.toLowerCase(), k = q.toLowerCase().trim(); if (!k) return 1;
  if (n === k) return 100; if (n.startsWith(k)) return 80; if (n.split(/[\s-]+/).some((w) => w.startsWith(k))) return 60; if (n.includes(k)) return 40;
  let j = 0; for (const ch of n) { if (ch === k[j]) j++; if (j === k.length) return 10; } return 0;
}
function linkSuggestions(c, lq) {
  if (lq.heading !== null) {
    const r = resolveLink(c, lq.query); if (r.kind !== 'note') return [];
    return mdRender(r.note.text, c).heads.filter((h) => fuzzyScore(h.text, lq.heading) > 0).slice(0, 8).map((h) => ({ kind: 'heading', label: h.text, sub: '#'.repeat(h.level) + ' in ' + r.note.name, target: r.note.name + '#' + h.text, insert: '[[' + r.note.name + '#' + h.text + ']]' }));
  }
  const seen = new Set(); const items = [];
  notesOf(c).forEach((n) => { const s = fuzzyScore(n.name, lq.query); if (s) { seen.add(n.name.toLowerCase()); items.push({ s: s + 5, kind: n.kind, label: n.name, sub: (NOTE_KINDS[n.kind] || NOTE_KINDS.note)[0] + ' note' + (n.private ? ' · private' : '') + (n.entity ? ' · on the map' : ''), target: n.name, insert: '[[' + n.name + ']]' }); } });
  linkEntities(c).forEach((e) => { if (seen.has(e.name.toLowerCase())) return; const s = fuzzyScore(e.name, lq.query); if (s) items.push({ s, kind: e.kind, label: e.name, sub: e.sub + ' · no note yet', target: e.name, insert: '[[' + e.name + ']]' }); });
  items.sort((a, b) => b.s - a.s || a.label.localeCompare(b.label));
  const out = items.slice(0, 8);
  const q = lq.query.trim(); if (q && !items.some((x) => x.label.toLowerCase() === q.toLowerCase())) out.push({ kind: 'create', label: 'Link to new note “' + q + '”', sub: 'created when you follow it', target: q, insert: '[[' + q + ']]' });
  return out;
}

/* ---------------- backlinks and unlinked mentions ---------------- */
function backlinksOf(c, note) {
  const k = note.name.toLowerCase(); const linked = [], unlinked = [];
  const re = /\[\[([^\]|#]+)/g;
  notesOf(c).forEach((n) => {
    if (n.id === note.id) return;
    n.text.split('\n').forEach((ln, i) => {
      let m, hit = false; re.lastIndex = 0; while ((m = re.exec(ln))) if (m[1].trim().toLowerCase() === k) hit = true;
      if (hit) { linked.push({ from: n, line: i, snippet: ln.replace(/^[#>\-\s*]+/, '').slice(0, 120) }); return; }
      const bare = ln.replace(/\[\[[^\]]*\]\]/g, '');
      const at = bare.toLowerCase().indexOf(k);
      if (at >= 0 && !/\w/.test(bare.charAt(at - 1) || ' ') && !/\w/.test(bare.charAt(at + k.length) || ' ')) unlinked.push({ from: n, line: i, snippet: ln.replace(/^[#>\-\s*]+/, '').slice(0, 120) });
    });
  });
  return { linked, unlinked };
}
function linkUnlinkedMention(c, note, hit) {
  const L = hit.from.text.split('\n'); const ln = L[hit.line];
  const re = new RegExp('(^|[^\\w\\[])(' + note.name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + ')(?![\\w\\]])', 'i');
  L[hit.line] = ln.replace(re, (m, a, b) => a + '[[' + note.name + (b !== note.name ? '|' + b : '') + ']]');
  saveNote(c, hit.from.id, L.join('\n'), 'linked a mention of ' + note.name);
}

/* ---------------- edits ---------------- */
function saveNote(c, id, text, why) {
  const notes = notesOf(c).map((n) => (n.id === id ? Object.assign({}, n, { text, edited: Date.now() }) : n));
  c.setState({ notes, neSaved: Date.now() }); if (why) c._toast(why);
}
function createNote(c, name, kind, entity) {
  const exist = noteByName(c, name); if (exist) return exist;
  const t = { settlement: '## Settlement Profile: ' + name + '\n\n**Type:** \n**Location:** \n\n### General Info\n\n- **Size / Population:**\n- **Ruling authority:**\n\n### Current Status\n\n', landmark: '# Landmark: ' + name + '\n\n## Type\n- \n\n## Description\n- Physical appearance:\n', faction: '# ' + name + '\n\n**Government:** \n**Faith:** \n\n## Relations\n\n' }[kind] || '# ' + name + '\n\n';
  const n = { id: 'n' + name.toLowerCase().replace(/[^a-z0-9]+/g, '-') + '-' + Math.random().toString(36).slice(2, 5), name, kind: kind || 'note', entity: entity || null, text: t, edited: Date.now(), private: false };
  c.setState({ notes: notesOf(c).concat([n]) });
  return n;
}
/* Toolbar transforms: each takes the text and a selection and returns the new text and selection, like the
   shipped editor's static transforms (`markdown_editor.gd`), so each is one undoable edit. */
const MD_TX = {
  wrap(t, a, b, open, close) { close = close == null ? open : close; const sel = t.slice(a, b); if (sel.startsWith(open) && sel.endsWith(close) && sel.length >= open.length + close.length) return { text: t.slice(0, a) + sel.slice(open.length, sel.length - close.length) + t.slice(b), a, b: b - open.length - close.length }; if (t.slice(a - open.length, a) === open && t.slice(b, b + close.length) === close) return { text: t.slice(0, a - open.length) + sel + t.slice(b + close.length), a: a - open.length, b: b - open.length }; const word = sel || 'text'; return { text: t.slice(0, a) + open + word + close + t.slice(b), a: a + open.length, b: a + open.length + word.length }; },
  linePrefix(t, a, b, pfx, cycle) {
    const s = t.lastIndexOf('\n', a - 1) + 1; let e = t.indexOf('\n', b); if (e < 0) e = t.length;
    const lines = t.slice(s, e).split('\n'); let d = 0;
    const out = lines.map((ln) => { if (cycle) { const m = /^(#{1,6})\s/.exec(ln); const lv = m ? m[1].length : 0; const nx = lv >= 3 ? 0 : lv + 1; const body = ln.replace(/^#{1,6}\s/, ''); const r = (nx ? '#'.repeat(nx) + ' ' : '') + body; d += r.length - ln.length; return r; } if (ln.startsWith(pfx)) { d -= pfx.length; return ln.slice(pfx.length); } const stripped = ln.replace(/^(- \[[ xX]\] |- |\d+\. |> )/, ''); const r = pfx + stripped; d += r.length - ln.length; return r; });
    return { text: t.slice(0, s) + out.join('\n') + t.slice(e), a: Math.max(s, a + (lines.length === 1 ? d : 0)), b: b + d };
  },
  insert(t, a, b, str, caretOff) { return { text: t.slice(0, a) + str + t.slice(b), a: a + (caretOff == null ? str.length : caretOff), b: a + (caretOff == null ? str.length : caretOff) }; }
};
function toggleTaskLine(c, note, line) {
  const L = note.text.split('\n'); const m = /^(\s*[-*+]\s+)\[([ xX])\]/.exec(L[line] || ''); if (!m) return;
  L[line] = L[line].replace(/^(\s*[-*+]\s+)\[([ xX])\]/, m[1] + (m[2] === ' ' ? '[x]' : '[ ]')); saveNote(c, note.id, L.join('\n'));
}
function wordCount(t) { const w = String(t).replace(/[#>*_=~`\[\]|-]/g, ' ').split(/\s+/).filter(Boolean); return w.length; }

/* A ```chronos block (the Chronos Timeline plugin's syntax, `cartalith_vault::chronos`, Ruling AM) drawn as a
   small timeline. Each row moves Cartalith's own timeline to its year, which is how a note's history and the
   map's timeline stay one thing. */
function chronosParse(lines) {
  const out = [];
  lines.forEach((ln) => {
    const m = /^\s*([-@*=])\s*\[([^\]~]+)(?:~([^\]]+))?\]\s*(?:#(\w+)\s*)?(?:\{([^}]*)\}\s*)?([^|]*)(?:\|\s*(.*))?$/.exec(ln);
    if (!m) return; const y0 = parseInt(m[2], 10), y1 = m[3] ? parseInt(m[3], 10) : null; if (isNaN(y0)) return;
    out.push({ kind: { '-': 'event', '@': 'period', '*': 'point', '=': 'marker' }[m[1]], y0, y1, color: m[4] || '', group: m[5] || '', name: m[6].trim(), desc: (m[7] || '').trim() });
  });
  return out;
}
function chronosHtml(lines, c) {
  const ev = chronosParse(lines); if (!ev.length) return '<pre><code>' + esc(lines.join('\n')) + '</code></pre>';
  const lo = Math.min(...ev.map((e) => e.y0)), hi = Math.max(...ev.map((e) => e.y1 != null ? e.y1 : e.y0)), span = Math.max(1, hi - lo);
  const pos = (y) => ((y - lo) / span * 100).toFixed(2) + '%';
  const bar = '<div class="md-chr-axis">' + ev.map((e) => e.kind === 'period' && e.y1 != null ? '<span class="md-chr-per" style="left:' + pos(e.y0) + ';width:calc(' + pos(e.y1) + ' - ' + pos(e.y0) + ')"></span>' : '<span class="md-chr-dot ' + e.kind + '" style="left:' + pos(e.y0) + '"></span>').join('') + '</div>';
  const rows = ev.map((e) => '<button class="md-chr-row" data-year="' + e.y0 + '"><span class="md-chr-y">' + esc(e.y0 + (e.y1 != null ? '–' + e.y1 : '')) + '</span><span class="md-chr-n">' + mdInline(e.name, c) + (e.desc ? '<span class="md-chr-d"> · ' + mdInline(e.desc, c) + '</span>' : '') + '</span><span class="md-chr-k">' + esc(e.kind) + '</span></button>').join('');
  return '<div class="md-chronos"><div class="md-co-t">CHRONOS · ' + ev.length + ' entries · tap one to move the map’s timeline</div>' + bar + rows + '</div>';
}
