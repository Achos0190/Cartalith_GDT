/* ======================================================================================================
   The vault and its note editor, on the phone and the tablet. Values only; the gestures are in c_class.js.
   Phone: a list (search, filters, kinds), then a note screen: Write by default (VQ4, Ruling DY), Preview one tap away, the keyboard
   row, links and outline in a sheet. Tablet: one window with the tree on the left, the note in the middle
   (Preview, Write or Split) and Links · Outline · Properties on the right.
   ====================================================================================================== */
const VT_FILTERS = [['all', 'All'], ['linked', 'On the map'], ['free', 'Free notes'], ['private', 'Private'], ['without', 'Without notes']];
const VT_GROUPS = [['settlement', 'Settlements'], ['province', 'Provinces'], ['faction', 'Factions'], ['culture', 'Cultures & faiths'], ['landmark', 'Landmarks'], ['note', 'Notes']];
/* The toolbar, in the shipped editor's order (`markdown_editor.gd`: bold, italic, underline, strike, highlight,
   heading, link, tag, task, callout, template) and then what a regular editor adds. */
const NE_TOOLS = [
  ['heading', 'H', 'Heading: cycles Normal, H1, H2, H3', ''], ['bold', 'B', 'Bold', 'Ctrl+B'], ['italic', 'I', 'Italic', 'Ctrl+I'], ['underline', 'U', 'Underline (written as <u>)', 'Ctrl+U'],
  ['strike', 'S', 'Strikethrough', ''], ['highlight', '==', 'Highlight', ''], ['link', '[[', 'Link a note or a place', 'Ctrl+K'], ['tag', '#', 'Tag', ''],
  ['bullet', '•', 'Bulleted list', ''], ['number', '1.', 'Numbered list', ''], ['task', '☐', 'Task', 'Ctrl+L'], ['quote', '❝', 'Quote', ''],
  ['callout', '[!]', 'Callout', ''], ['code', '</>', 'Code', ''], ['table', '⊞', 'Table', ''], ['rule', '—', 'Divider', ''],
  ['template', '⎘', 'Insert template', ''], ['undo', '↶', 'Undo', 'Ctrl+Z'], ['redo', '↷', 'Redo', 'Ctrl+Shift+Z'], ['revert', '⟲', 'Revert to opened…', '']
];
function vtAgo(ts) { const d = Math.max(0, Date.now() - ts); if (d < 60000) return 'just now'; if (d < 3600000) return Math.round(d / 60000) + ' min ago'; if (d < 86400000) return Math.round(d / 3600000) + ' h ago'; return Math.round(d / 86400000) + ' d ago'; }
function vtPath(n) { const g = { settlement: 'Settlements', province: 'Provinces', faction: 'Factions', culture: 'Cultures', landmark: 'Landmarks' }[n.kind] || 'Notes'; return 'Vault/' + g + '/' + n.name + '.md'; }
function vtTags(t) { const out = []; String(t).replace(/(^|\s)#([A-Za-z][\w/-]*)/g, (m, a, b) => { if (!out.includes(b)) out.push(b); return m; }); return out; }

/* The list: notes filtered by the search (name and full text) and the filter chip, grouped by kind. "Without
   notes" lists world entities that have no note; tapping one creates its note from the matching template. */
function vtList(c) {
  const s = c.state, V = s.vt, q = (V.q || '').trim().toLowerCase();
  if (V.filter === 'without') {
    const have = new Set(notesOf(c).map((n) => n.name.toLowerCase()));
    const ents = linkEntities(c).filter((e) => !have.has(e.name.toLowerCase()) && (!q || e.name.toLowerCase().includes(q)));
    return VT_GROUPS.map(([k, label]) => ({ label: label.toUpperCase(), n: ents.filter((e) => e.kind === k).length, rows: ents.filter((e) => e.kind === k).slice(0, 12).map((e) => ({ label: e.name, sub: e.sub, d: (NOTE_KINDS[e.kind] || NOTE_KINDS.note)[1], right: 'CREATE', on: false, bg: 'transparent', lock: false, tap: () => c._noteCreate(e.name, e.kind, e) })) })).filter((g) => g.n);
  }
  const hit = (n) => !q || n.name.toLowerCase().includes(q) || n.text.toLowerCase().includes(q) || vtTags(n.text).some((t) => ('#' + t).includes(q.replace(/^#?/, '#')));
  const pass = (n) => V.filter === 'all' || (V.filter === 'linked' && n.entity) || (V.filter === 'free' && !n.entity) || (V.filter === 'private' && n.private);
  const notes = notesOf(c).filter((n) => hit(n) && pass(n));
  return VT_GROUPS.map(([k, label]) => {
    const rows = notes.filter((n) => (n.kind || 'note') === k).sort((a, b) => a.name.localeCompare(b.name)).map((n) => {
      let sub = (n.entity ? '⌖ on the map · ' : '') + 'edited ' + vtAgo(n.edited);
      if (q && !n.name.toLowerCase().includes(q)) { const ln = n.text.split('\n').find((x) => x.toLowerCase().includes(q)) || ''; sub = '… ' + ln.replace(/[#>*=\[\]-]/g, '').trim().slice(0, 60); }
      return { label: n.name, sub, d: (NOTE_KINDS[n.kind] || NOTE_KINDS.note)[1], right: '', on: V.noteId === n.id, bg: V.noteId === n.id ? 'var(--wash)' : 'transparent', lock: !!n.private, tap: () => c._noteOpen(n.id) };
    });
    return { label: label.toUpperCase(), n: rows.length, rows };
  }).filter((g) => g.n);
}

function vtNote(c) { const V = c.state.vt; return V.noteId ? noteById(c, V.noteId) : null; }

function vaultVals(c) {
  const s = c.state, V = s.vt; const m = c._mode(), W = c._W(), H = c._H();
  if (!V || !V.open) return { show: false, groups: [], filters: [], tools: [], ac: { show: false, items: [] }, links: [], unlinked: [], heads: [], props: [], chips: [], dlg: { show: false, opts: [] } };
  const phone = m !== 'tablet', note = vtNote(c);
  const groups = vtList(c);
  const filters = VT_FILTERS.map(([id, label]) => Object.assign({ label, tap: () => c.setState({ vt: Object.assign({}, V, { filter: id }) }) }, chipStyle(V.filter === id)));
  /* geometry */
  const frame = phone ? { x: 0, y: 0, w: W, h: H, r: 0 } : { x: 20, y: 20, w: W - 40, h: H - 40, r: 20 };
  const sideW = phone ? W : (frame.w >= 1100 ? 270 : 240);
  const panelOn = !phone && V.panel !== 'none' && frame.w - sideW >= 900;
  const panelW = panelOn ? 290 : 0;
  const sideOn = phone ? V.view === 'list' : V.side !== false;
  const noteW = phone ? W : frame.w - (sideOn ? sideW : 0) - panelW;
  const splitOk = noteW >= 760;
  const mode = V.mode === 'split' && !splitOk ? 'write' : V.mode;
  /* the note */
  let n = { show: false };
  if (note) {
    const text = V.buf != null ? V.buf : note.text;
    const r = mdRender(text, c); const bl = backlinksOf(c, note); const tags = vtTags(text);
    const ent = note.entity || null;
    const chips = [{ label: (NOTE_KINDS[note.kind] || NOTE_KINDS.note)[0].toLowerCase(), key: 'type' }].concat(ent ? [{ label: ent.name, key: 'on the map', tap: () => c._noteShowOnMap(note) }] : []).concat(tags.slice(0, 4).map((t) => ({ label: '#' + t, key: 'tag', tap: () => c.setState({ vt: Object.assign({}, V, { view: 'list', q: '#' + t, filter: 'all' }) }) }))).concat(note.private ? [{ label: 'private', key: 'lock' }] : []);
    n = {
      show: true, id: note.id, title: note.name, path: vtPath(note), mode, splitOk,
      read: mode === 'read' || mode === 'split', write: mode === 'write' || mode === 'split',
      text, words: wordCount(text) + ' words', saved: V.dirty ? 'editing…' : (s.neSaved && Date.now() - s.neSaved < 4000 ? 'saved' : 'saved · ' + vtAgo(note.edited)),
      chips: chips.map((x) => Object.assign({ tap: () => {} }, x, { cur: x.tap ? 'pointer' : 'default' })),
      modes: [['read', 'Preview'], ['write', 'Write']].concat(splitOk ? [['split', 'Split']] : []).map(([id, label]) => Object.assign({ label, tap: () => c._noteMode(id), aria: label + ' view' }, chipStyle(mode === id))),
      heads: r.heads.map((h) => ({ label: h.text, pad: (h.level - 1) * 14 + 'px', tap: () => c._noteJump(h) })),
      links: bl.linked.map((h) => ({ label: h.from.name, sub: h.snippet, tap: () => c._noteOpen(h.from.id) })),
      unlinked: bl.unlinked.map((h) => ({ label: h.from.name, sub: h.snippet, tap: () => c._noteOpen(h.from.id), link: () => { linkUnlinkedMention(c, note, h); } })),
      nLinks: bl.linked.length, nUnlinked: bl.unlinked.length,
      outgoing: (text.match(/\[\[[^\]]+\]\]/g) || []).length,
      ent: ent ? { name: ent.name, sub: ({ settlement: 'settlement', faction: 'faction · its territory', landmark: 'landmark' }[ent.kind] || ent.kind), map: () => c._noteShowOnMap(note) } : null,
      conflict: !!V.conflict,
      phoneLinksOpen: !!V.sheet,
      panel: V.panel || 'links'
    };
    n.hasEnt = !!n.ent; n.entName = ent ? ent.name : ''; n.entSub = n.ent ? n.ent.sub : '';
    n.panelTabs = [['links', 'Links ' + (bl.linked.length + bl.unlinked.length)], ['outline', 'Outline'], ['props', 'Properties']].map(([id, label]) => Object.assign({ label, tap: () => c.setState({ vt: Object.assign({}, V, { panel: id, sheet: phone ? id : V.sheet }) }) }, chipStyle((phone ? V.sheet : (V.panel || 'links')) === id)));
    n.props = [['File', vtPath(note)], ['Kind', (NOTE_KINDS[note.kind] || NOTE_KINDS.note)[0]], ['On the map', ent ? ent.name + ' (' + ent.kind + ')' : 'not linked'], ['Link status', ent ? (V.conflict ? 'stale: changed outside' : (V.dirty ? 'local changes' : 'connected')) : 'unbound'], ['Tags', tags.length ? tags.map((t) => '#' + t).join(' ') : 'none'], ['Links out', String(n.outgoing)], ['Edited', vtAgo(note.edited)]].map(([k, v]) => ({ k, v }));
    n.privTog = { on: !!note.private, tap: () => c._notePrivate(note.id) }; n.privLabel = note.private ? 'ON' : 'OFF'; n.outgoingLabel = n.outgoing + ' link' + (n.outgoing === 1 ? '' : 's') + ' out';
    if (!n.ent) n.ent = { name: '', sub: '', map: () => {} };
  }
  /* [[ suggestions */
  const ac = s.ac && s.ac.items && s.ac.items.length ? { show: true, items: s.ac.items.map((it, i) => ({ label: it.label, sub: it.sub, d: it.kind === 'create' ? IC.star : it.kind === 'heading' ? IC.notes : (NOTE_KINDS[it.kind] || NOTE_KINDS.note)[1], on: i === s.ac.sel, bg: i === s.ac.sel ? 'var(--wash)' : 'transparent', keep: (e) => e.preventDefault(), tap: () => c._neAcPick(i) })) } : { show: false, items: [] };
  /* toolbar */
  const tools = NE_TOOLS.map(([id, glyph, label, key]) => ({ id, glyph, aria: label + (key ? ' (' + key + ')' : ''), keep: (e) => e.preventDefault(), tap: () => c._neTx(id) }));
  /* dialogs: create from template, rename */
  const D = s.neDlg; let dlg = { show: false, opts: [] };
  if (D && D.kind === 'create') dlg = { show: true, title: 'NEW NOTE · ' + D.name.toUpperCase(), text: D.entity ? D.entity.name + ' is on the map (' + D.entity.sub + ') and has no note yet.' : 'No note is called “' + D.name + '” yet.', opts: [['Settlement', 'settlement'], ['Landmark', 'landmark'], ['Faction', 'faction'], ['Region overview', 'region'], ['Blank', 'note']].map(([label, k]) => Object.assign({ label, tap: () => c._noteCreateDo(D.name, k, D.entity) }, chipStyle(k === (D.entityKind || 'note')))), map: D.entity ? () => { c.setState({ neDlg: null }); c._noteShowOnMap({ entity: D.entity, name: D.entity.name }); } : null };
  if (D && D.kind === 'template') dlg = { show: true, title: 'INSERT TEMPLATE', text: 'Inserted at the cursor. The vault’s own templates (the ones in its templates folder) are listed first in the real app.', opts: [['Settlement · general info', 'settlement'], ['Landmark · significance', 'landmark'], ['Faction · relations table', 'faction'], ['Chronos timeline block', 'chronos']].map(([label, k]) => Object.assign({ label, tap: () => c._neTemplate(k) }, chipStyle(false))) };
  if (D && D.kind === 'rename') dlg = { show: true, title: 'RENAME NOTE', text: D.count ? D.count + ' link' + (D.count === 1 ? '' : 's') + ' in ' + D.notes + ' note' + (D.notes === 1 ? '' : 's') + ' will be rewritten to the new name, keeping aliases and heading links.' : 'No other note links here.', opts: [], rename: true, value: D.value };
  dlg.hasMap = !!dlg.map; dlg.isRename = !!dlg.rename;
  /* peek card */
  const P = s.peek; let peek = { show: false };
  if (P) { const r = resolveLink(c, P.name); const body = r.kind === 'note' ? r.note.text.split('\n').filter((x) => x.trim() && !/^#/.test(x)).slice(0, 3).join(' ').replace(/[*=~#>\[\]_`|]/g, '').slice(0, 180) : r.kind === 'entity' ? 'On the map, no note yet.' : 'No note or place has this name yet.'; peek = { show: true, x: Math.max(8, Math.min(W - 308, P.x - 150)), y: Math.max(8, Math.min(H - 190, P.y + 16)), title: P.name, kind: r.kind === 'note' ? (NOTE_KINDS[r.note.kind] || NOTE_KINDS.note)[0] + ' note' : r.kind === 'entity' ? r.entity.sub : 'unresolved link', body, openLabel: r.kind === 'note' ? 'Open' : 'Create note', open: () => { c.setState({ peek: null }); c._noteFollow(P.name, P.head); }, hasMap: r.kind !== 'none' && !!(r.entity || (r.note && r.note.entity)), map: () => { c.setState({ peek: null }); c._noteShowOnMap(r.note || { entity: r.entity, name: r.entity.name }); } }; }
  const land = m === 'phoneLand';
  const acOn = ac.show && !!note && (mode === 'write' || mode === 'split');
  return {
    true: true, false: false, scrim: !phone, padT: phone ? (land ? 8 : INSET_T) : 0, padB: phone ? 16 : 0,
    panelAria: phone ? 'links, outline and properties' : (panelOn ? 'hide the side panel' : 'show links, outline and properties'),
    splitLine: mode === 'split' ? 'var(--hair)' : 'transparent', linksBelow: !!note && (phone || !panelOn) && mode === 'read',
    acTop: acOn && !phone, acTopY: 158, acBottom: acOn && phone,
    show: true, phone, frame, sideW, sideOn, panelOn, panelW, noteW, noteX: phone ? 0 : (sideOn ? sideW : 0),
    list: phone ? V.view === 'list' : true, noteShow: !!note && (phone ? V.view === 'note' : true), empty: !note && !phone,
    q: V.q || '', count: notesOf(c).length + ' notes', filters, groups, n, ac, tools, dlg, peek,
    sheetOpen: phone && !!V.sheet && !!note, sheetIs: { links: V.sheet === 'links', outline: V.sheet === 'outline', props: V.sheet === 'props' },
    panelIs: { links: (V.panel || 'links') === 'links', outline: V.panel === 'outline', props: V.panel === 'props' },
    canBack: phone ? true : (V.back || []).length > 0, back: () => c._noteBack(),
    toolsBottom: phone && !!note && V.view === 'note' && (n.write), toolsTop: !phone && !!note && n.write,
    chip: !!note && !!n.write && !c._ptr() && !!(s.caretLink && s.caretLink.where === 'vault'), chipLabel: s.caretLink ? 'Open ' + s.caretLink.name + ' ›' : '', chipAria: s.caretLink ? 'open the link at the caret: ' + s.caretLink.name : '', chipTap: () => s.caretLink && c._followFromWrite(s.caretLink.name, 'vault'), keep: (e) => e.preventDefault(),
    taTitle: c._ptr() ? 'Markdown · Ctrl+click a link to open it · Ctrl+E preview' : 'Markdown · put the caret in a link to open it'
  };
}
