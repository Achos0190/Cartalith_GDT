/* ======================================================================================================
   The eight program menus, row for row as `menus.gd` builds them on the desktop (read 2026-10-31).
   The PC menu bar is unchanged by this mockup: the phone's MENU tab and the tablet's menu bar present
   these same rows, in the same order, under the same names. Row kinds keep the shell's vocabulary:
     live      an enabled command               todo     drawn, does nothing, says why
     readout   a disabled row carrying a value  sign     prose pointing somewhere else
     check / radio / sub (submenu) / band (captioned separator) / sep
   `act` names what the mockup does when the row is used; `win:<id>` opens that window.
   ====================================================================================================== */
const STAGE_NAMES = ['Planet', 'Extent & scale', 'World structure', 'Tectonics', 'Volcanism & impacts', 'Erosion', 'Hydrology', 'Climate', 'Ecology & biomes', 'Resources & soils'];
const LANDMARK_FAMILIES = [
  ['Physical', ['Peak', 'Ridge', 'Saddle*', 'Cliff', 'Gorge', 'Cave', 'Waterfall', 'Spring', 'Lake', 'Delta*', 'River confluence', 'Volcanic feature', 'Rock formation', 'Glacial feature*', 'Ancient forest*']],
  ['Transport', ['Mountain pass', 'River crossing*', 'Ford', 'Bridge site', 'Road junction', 'Caravan station', 'Portage', 'Harbour']],
  ['Economic', ['Mine', 'Quarry', 'Salt works*', 'Resource extraction site', 'Market site', 'Trade depot']],
  ['Military', ['Fort', 'Watchtower', 'Fortified pass', 'Fortified crossing', 'Battlefield', 'Border marker']],
  ['Cultural', ['Shrine*', 'Temple*', 'Sacred grove*', 'Sacred mountain*', 'Pilgrimage site*', 'Tomb*', 'Monument*', 'Ceremonial site*']],
  ['Historical', ['Ruin*', 'Abandoned settlement', 'Ancient road', 'Historic battlefield*', 'Destroyed fortress*', 'Historic crossing*']]
];
const ASSET_FAMILIES = [['Splat channels', 'texture', 4, 4], ['Biome textures', 'texture', 9, 12], ['Terrain textures', 'texture', 6, 8], ['Feature icons', 'icon', 22, 31], ['Settlement pins', 'icon', 8, 10], ['Settlement traits', 'icon', 5, 12], ['Points of interest', 'icon', 14, 21], ['Sea marks', 'icon', 4, 9], ['Custom icons', 'icon', 0, 6]];

function menuTree() {
  const lmFamilies = LANDMARK_FAMILIES.map(([fam, kinds]) => ({
    t: 'sub', label: fam, sub: 'armed · placed · caps', items: kinds.map((k) => k.endsWith('*') ? { t: 'live', label: k.slice(0, -1) + '   not buildable', act: 'node', arg: 'civil/landmarks' } : { t: 'live', label: k, act: 'lmType', arg: k }).concat([{ t: 'sep' }, { t: 'live', label: 'Open ' + fam + ' in the dock', act: 'node', arg: 'civil/landmarks' }, { t: 'sign', label: 'Rows here read state; they do not arm a type.' }])
  }));
  const stages = STAGE_NAMES.map((n, i) => ({ t: 'live', label: String(i + 1).padStart(2, '0') + ' ' + n, act: 'resetStage', arg: i + 1, sm: true }));
  return [
    { id: 'file', label: 'File', d: IC.file, sub: 'new · open · save · export · storage', items: [
      { t: 'live', label: 'New world…', accel: 'Ctrl+N', act: 'newWorld', sm: true },
      { t: 'live', label: 'Generate world', accel: 'Ctrl+R', act: 'run', sm: true },
      { t: 'live', label: 'Open project…', accel: 'Ctrl+O', act: 'picker' },
      { t: 'sub', label: 'Recent worlds', items: [{ t: 'live', label: 'VHAREN REACH — 129384 · 5 d ago', act: 'openWorld', arg: 'VHAREN REACH' }, { t: 'live', label: 'KESSA — 774201 · 3 w ago', act: 'openWorld', arg: 'KESSA' }, { t: 'live', label: 'THORNWOOD — 908133 · 2 mo ago', act: 'openWorld', arg: 'THORNWOOD' }] },
      { t: 'live', label: 'New world from selection…', act: 'fromSelection', sm: true },
      { t: 'sep' },
      { t: 'live', label: 'Save project', accel: 'Ctrl+S', act: 'save' },
      { t: 'live', label: 'Save as…', accel: 'Ctrl+Shift+S', act: 'saveAs' },
      { t: 'check', label: 'Autosave', key: 'prefs.autosave' },
      { t: 'sub', label: 'Autosave interval', items: [{ t: 'radio', label: 'Off', key: 'prefs.autoInt', value: 0 }, { t: 'radio', label: 'Every 1 min', key: 'prefs.autoInt', value: 1 }, { t: 'radio', label: 'Every 5 min', key: 'prefs.autoInt', value: 5 }, { t: 'radio', label: 'Every 15 min', key: 'prefs.autoInt', value: 15 }] },
      { t: 'live', label: 'Revert to last save', act: 'revert' },
      { t: 'live', label: 'Close project', accel: 'Ctrl+W', act: 'close' },
      { t: 'sep' },
      { t: 'sub', label: 'Export', items: [
        { t: 'band', label: 'FOR THE TABLE' },
        { t: 'live', label: 'Player map…', accel: 'Ctrl+Shift+E', act: 'win:export', arg: 'player' },
        { t: 'live', label: 'DM / full map…', act: 'win:export', arg: 'dm' },
        { t: 'band', label: 'SHARE' },
        { t: 'live', label: 'Export map image…', act: 'win:export', arg: 'image' },
        { t: 'todo', label: 'Print sheet (PDF)…', reason: 'No print or PDF writer exists in this build.' },
        { t: 'todo', label: 'VTT battle map (grid + walls)…', reason: 'No virtual-tabletop format exists in this build: no grid calibration, wall or light data is computed or written.' },
        { t: 'band', label: 'DATA' },
        { t: 'live', label: 'Maps   tiles', act: 'win:datamanager', arg: 'export-maps' },
        { t: 'live', label: 'GIS / GeoJSON   .geojson', act: 'win:datamanager', arg: 'export-gis' },
        { t: 'live', label: 'World Data   map + atlas', act: 'win:datamanager', arg: 'export-world' },
        { t: 'live', label: 'Assets   .zip', act: 'win:assets' }
      ] },
      { t: 'band', label: 'STORAGE LOCATIONS' },
      { t: 'readout', label: 'projects', v: '…/Cartalith/Worlds' },
      { t: 'readout', label: 'tile atlas cache', v: '…/Cartalith/Atlas' },
      { t: 'readout', label: 'asset packs', v: '…/Cartalith/Packs' },
      { t: 'readout', label: 'exports', v: '…/Cartalith/Exports' },
      { t: 'live', label: 'Change locations…', act: 'win:storage' },
      { t: 'live', label: 'Show project on disk', act: 'showOnDisk', phone: 'Share project…' },
      { t: 'sign', label: 'imports live under Records ▸ Import' },
      { t: 'sign', label: 'asset packs under Assets' }
    ] },
    { id: 'edit', label: 'Edit', d: IC.edit, sub: 'undo · clipboard · find · reset', items: [
      { t: 'live', label: 'Undo', accel: 'Ctrl+Z', act: 'undo', dyn: 'undo' },
      { t: 'live', label: 'Redo', accel: 'Ctrl+Shift+Z', act: 'redo', dyn: 'redo' },
      { t: 'live', label: 'Undo history…', act: 'history' },
      { t: 'sep' },
      { t: 'live', label: 'Cut', accel: 'Ctrl+X', act: 'clip', arg: 'Cut' },
      { t: 'live', label: 'Copy', accel: 'Ctrl+C', act: 'clip', arg: 'Copy' },
      { t: 'live', label: 'Paste', accel: 'Ctrl+V', act: 'clip', arg: 'Paste' },
      { t: 'live', label: 'Delete', accel: 'Delete', act: 'delete' },
      { t: 'sep' },
      { t: 'live', label: 'Select all', accel: 'Ctrl+A', act: 'selectAll' },
      { t: 'live', label: 'Deselect', accel: 'Ctrl+D', act: 'deselect' },
      { t: 'live', label: 'Previous selection', accel: 'Alt+←', act: 'selBack' },
      { t: 'live', label: 'Next selection', accel: 'Alt+→', act: 'selFwd' },
      { t: 'sep' },
      { t: 'live', label: 'Find on map…', accel: 'Ctrl+F', act: 'search' },
      { t: 'live', label: 'Context card', accel: 'Shift+F10', act: 'ctxCard', notPhone: true },
      { t: 'live', label: 'Cartwheel at the pointer', accel: 'Q', act: 'ringHere', notPhone: true },
      { t: 'sep' },
      { t: 'live', label: 'Reset generation parameters', act: 'resetParams', sm: true },
      { t: 'sub', label: 'Reset one stage', sm: true, items: stages }
    ] },
    { id: 'assets', label: 'Assets', d: IC.assets, sub: 'library · packs · families · landmark types', items: [
      { t: 'live', label: '⧉ Asset library', accel: 'Shift+A', act: 'win:assets' },
      { t: 'live', label: '⧉ Sprite sheet slicer (▦)', act: 'win:assets', arg: 'slicer' },
      { t: 'live', label: 'Import asset pack .zip…', act: 'filePick', arg: 'asset pack .zip' },
      { t: 'live', label: 'Open pack in library…', act: 'filePick', arg: 'pack (library only)' },
      { t: 'live', label: 'Pack report…', act: 'win:packreport' },
      { t: 'live', label: 'Import folder of art…', act: 'filePick', arg: 'folder of art' },
      { t: 'live', label: 'Compare pack with library…', act: 'filePick', arg: 'pack to compare' },
      { t: 'sep' },
      { t: 'sub', label: 'Asset pack', items: [
        { t: 'band', label: 'ACTIVE PACK' },
        { t: 'readout', label: 'NAME', v: 'Eldra base' }, { t: 'readout', label: 'AUTHOR', v: 'you' }, { t: 'readout', label: 'LICENSE', v: 'CC BY 4.0' }, { t: 'readout', label: 'SCHEMA', v: '2 · STORED zip' }, { t: 'readout', label: 'FILLED', v: '72 of 113 slots · 214 items' },
        { t: 'live', label: 'Pack metadata…', act: 'win:packmeta' },
        { t: 'live', label: 'Validate pack', act: 'toast', arg: 'Pack validated · 0 errors · 3 warnings (read-only)' },
        { t: 'live', label: 'Export pack .zip…', accel: 'Ctrl+Shift+P', act: 'filePick', arg: 'export destination' }
      ] },
      { t: 'sub', label: 'Icon families', items: ASSET_FAMILIES.filter((f) => f[1] === 'icon').map((f) => ({ t: 'live', label: f[0] + '   ' + f[2] + '/' + f[3], act: 'win:assets', arg: f[0] })) },
      { t: 'sub', label: 'Texture sets', items: ASSET_FAMILIES.filter((f) => f[1] === 'texture').map((f) => ({ t: 'live', label: f[0] + '   ' + f[2] + '/' + f[3], act: 'win:assets', arg: f[0] })) },
      { t: 'sub', label: 'Landmark types', items: lmFamilies.concat([{ t: 'live', label: 'Placed by hand   3', act: 'node', arg: 'civil/landmarks' }, { t: 'sep' }, { t: 'live', label: 'Landmark icons…', act: 'win:assets', arg: 'Points of interest' }, { t: 'live', label: 'Landmark label style…', act: 'node', arg: 'carto/labels' }]) },
      { t: 'sep' },
      { t: 'live', label: 'Apply library to map', act: 'toast', arg: 'Library applied to the map · 72 slots drawn' },
      { t: 'live', label: 'Clear library…', danger: true, act: 'confirm', arg: 'Clear the asset library? The map keeps its pins until you apply.' }
    ] },
    { id: 'records', label: 'Records', d: IC.vault, sub: 'data manager · records · technical', items: [
      { t: 'live', label: '⧉ Data manager', act: 'win:datamanager' },
      { t: 'band', label: 'RECORDS' },
      { t: 'live', label: 'Factions…', accel: 'Ctrl+Shift+F', act: 'factionsHere' },
      { t: 'live', label: 'Places…', act: 'node', arg: 'civil/settle' },
      { t: 'live', label: 'Markdown vault…', accel: 'Ctrl+Shift+V', act: 'win:vault' },
      { t: 'live', label: 'Browse & edit a note…', act: 'win:vault', arg: 'browse' },
      { t: 'live', label: 'Places without notes…', act: 'win:vault', arg: 'without' },
      { t: 'sign', label: 'Timeline is in Story ▸ Timeline' },
      { t: 'band', label: 'TECHNICAL' },
      { t: 'live', label: 'World data tables…', act: 'win:worlddata' },
      { t: 'sub', label: 'Import', items: [{ t: 'live', label: 'Maps   tiles', act: 'win:datamanager', arg: 'import-maps' }, { t: 'live', label: 'Heightmaps   PNG', act: 'win:datamanager', arg: 'import-height' }, { t: 'live', label: 'GIS / GeoJSON', act: 'win:datamanager', arg: 'import-gis' }, { t: 'live', label: 'World Data   .ctl', act: 'picker' }, { t: 'live', label: 'Assets   → Assets', act: 'win:assets' }] },
      { t: 'sub', label: 'Sources & validation', items: [{ t: 'band', label: 'SOURCES' }, { t: 'live', label: 'External Sources', act: 'win:datamanager', arg: 'src-ext' }, { t: 'live', label: 'Connected Sources', act: 'win:datamanager', arg: 'src-conn' }, { t: 'live', label: 'Source Registry', act: 'win:datamanager', arg: 'src-reg' }, { t: 'band', label: 'VALIDATION' }, { t: 'live', label: 'Definitions', act: 'win:datamanager', arg: 'val-def' }, { t: 'live', label: 'Check Data', act: 'win:datamanager', arg: 'val-check' }, { t: 'live', label: 'Repair / Normalize', act: 'win:datamanager', arg: 'val-repair' }] },
      { t: 'sign', label: 'Export is in File ▸ Export' }
    ] },
    { id: 'story', label: 'Story', d: IC.story, sub: 'time · journeys · conflicts · notes', items: [
      { t: 'band', label: 'TIME' },
      { t: 'live', label: 'Timeline', accel: 'Ctrl+Shift+T', act: 'story', arg: 'time' },
      { t: 'live', label: 'Political history of the selection', act: 'ctx', arg: 'inspect', needSel: true },
      { t: 'live', label: 'Faction history…', act: 'factionsHere', arg: 'history' },
      { t: 'band', label: 'JOURNEYS' },
      { t: 'live', label: 'Journey planner', act: 'story', arg: 'journeys' },
      { t: 'live', label: 'Saved journeys…', act: 'win:journeys' },
      { t: 'live', label: 'Travel library…', accel: 'Shift+L', act: 'win:travel' },
      { t: 'band', label: 'CONFLICTS & CAMPAIGNS' },
      { t: 'live', label: 'Conflicts…', act: 'story', arg: 'conflicts' },
      { t: 'live', label: 'Draw a conflict', act: 'tool', arg: 'conflict' },
      { t: 'readout', label: 'Campaigns this year', dyn: 'campaigns' },
      { t: 'band', label: 'NOTES' },
      { t: 'live', label: 'Note for the selection', accel: 'Ctrl+Shift+N', act: 'ctx', arg: 'note', needSel: true },
      { t: 'sign', label: 'Places without notes… is in Records' },
      { t: 'live', label: 'Ask the assistant about the selection', accel: 'Ctrl+J', act: 'ctx', arg: 'ask' }
    ] },
    { id: 'prefs', label: 'Preferences', d: IC.gear, sub: 'performance · graphics · LOD · memory · app', items: [
      { t: 'band', label: 'PERFORMANCE' },
      { t: 'check', label: 'GPU acceleration', key: 'prefs.gpu', note: 'Vulkan' },
      { t: 'sub', label: 'Devices', items: [{ t: 'check', label: 'Automatic (highest-performance GPU)', key: 'prefs.gpuAuto', on: true }, { t: 'sep' }, { t: 'check', label: 'Adreno 630 · integrated · Vulkan', key: 'prefs.gpuDev', on: true }, { t: 'sep' }, { t: 'readout', label: 'Memory', v: 'not measured yet' }, { t: 'sep' }, { t: 'live', label: 'Rescan devices…', act: 'toast', arg: 'Rescanned · 1 adapter' }] },
      { t: 'sub', label: 'Multi-GPU mode', items: [{ t: 'radio', label: 'Single device', key: 'prefs.multi', value: 'single' }, { t: 'radio', label: 'Split tiles', key: 'prefs.multi', value: 'split' }, { t: 'todo', label: 'Alternate frames', reason: 'Not built.' }] },
      { t: 'sub', label: 'CPU worker threads', items: [{ t: 'radio', label: 'Automatic', key: 'prefs.thr', value: 'auto' }, { t: 'radio', label: '1 — 1 thread', key: 'prefs.thr', value: 1 }, { t: 'radio', label: '2 — quarter', key: 'prefs.thr', value: 2 }, { t: 'radio', label: '4 — half', key: 'prefs.thr', value: 4 }, { t: 'radio', label: '8 — all cores', key: 'prefs.thr', value: 8 }, { t: 'readout', label: 'Running', v: '8 workers now' }] },
      { t: 'sub', label: 'VRAM budget', items: ['No cap', '1 GB', '2 GB', '4 GB', '8 GB', '12 GB', '16 GB', '24 GB'].map((x) => ({ t: 'radio', label: x, key: 'prefs.vram', value: x })).concat([{ t: 'readout', label: 'Current grid 2048×2048 needs about', v: '412 MB' }]) },
      { t: 'sub', label: 'Fallback when VRAM full', items: [{ t: 'radio', label: 'CPU tile pass', key: 'prefs.fallback', value: 'cpu' }, { t: 'todo', label: 'Reduce working res', reason: 'Not built.' }, { t: 'radio', label: 'Fail with error', key: 'prefs.fallback', value: 'fail' }] },
      { t: 'live', label: 'Try the GPU again', act: 'toast', arg: 'GPU readback cleared · the next run tries the GPU', dyn: 'gpuRetry' },
      { t: 'band', label: 'GRAPHICS' },
      { t: 'sub', label: 'Render quality', items: ['performance', 'balanced', 'quality', 'ultra'].map((x) => ({ t: 'radio', label: x, key: 'prefs.quality', value: x })) },
      { t: 'todo', label: 'Anti-aliasing · anisotropy', reason: 'The 3D view draws no terrain yet, so there is nothing to anti-alias.' },
      { t: 'sign', label: 'Colour management — CARTO ▸ Colours ▸ Colour management' },
      { t: 'todo', label: '3D viewport defaults', reason: 'They arrive with the 3D view’s terrain.' },
      { t: 'sub', label: 'Lighting rig defaults', items: [{ t: 'readout', label: 'Following the engine', v: '' }, { t: 'sub', label: 'Azimuth', items: [0, 45, 90, 135, 180, 225, 270, 315].map((x) => ({ t: 'radio', label: x + ' deg', key: 'prefs.az', value: x })) }, { t: 'sub', label: 'Elevation', items: [15, 25, 35, 45, 55, 65, 75].map((x) => ({ t: 'radio', label: x + ' deg', key: 'prefs.alt', value: x })) }, { t: 'sub', label: 'Ambient', items: ['0.00', '0.15', '0.25', '0.35', '0.50', '0.70'].map((x) => ({ t: 'radio', label: x, key: 'prefs.amb', value: x })) }, { t: 'sub', label: 'Multidirectional', items: ['1 light (single sun)', '2 lights', '4 lights', '6 lights', '8 lights', '12 lights'].map((x) => ({ t: 'radio', label: x, key: 'prefs.lights', value: x })) }, { t: 'sep' }, { t: 'live', label: 'Use the engine’s own rig', act: 'toast', arg: 'Lighting follows the engine again' }] },
      { t: 'sub', label: 'Relief exaggeration', items: [{ t: 'radio', label: '1x', key: 'prefs.relief', value: '1x' }, { t: 'radio', label: '2x', key: 'prefs.relief', value: '2x' }, { t: 'radio', label: '4x', key: 'prefs.relief', value: '4x' }, { t: 'sep' }, { t: 'live', label: 'Use the engine’s own 3.4x', act: 'toast', arg: 'Relief exaggeration follows the engine (3.4x)' }] },
      { t: 'band', label: 'TILES & LOD' },
      { t: 'sub', label: 'Tiled LOD', items: [{ t: 'radio', label: 'Auto on zoom', key: 'prefs.lodMode', value: 'auto' }, { t: 'radio', label: 'Manual', key: 'prefs.lodMode', value: 'manual' }, { t: 'sep' }, { t: 'live', label: 'Enter deep detail now', act: 'toast', arg: 'Deep detail on' }, { t: 'live', label: 'Leave deep detail', act: 'toast', arg: 'Deep detail off' }] },
      { t: 'sub', label: 'Tile size · LOD levels', items: [{ t: 'radio', label: '256 px', key: 'prefs.tile', value: 256 }, { t: 'radio', label: '512 px', key: 'prefs.tile', value: 512 }, { t: 'radio', label: '1024 px', key: 'prefs.tile', value: 1024 }, { t: 'sep' }, { t: 'sub', label: 'LOD levels', items: [['0–0', '1'], ['0–1', '5'], ['0–2', '21'], ['0–3', '85'], ['0–4', '341'], ['0–5', '1 365'], ['0–6', '5 461'], ['0–7', '21 845'], ['0–8', '87 381']].map((x) => ({ t: 'radio', label: 'LOD ' + x[0] + '   ' + x[1] + (x[1] === '1' ? ' tile' : ' tiles'), key: 'prefs.lod', value: x[0] })) }] },
      { t: 'sub', label: 'Atlas cache', items: [{ t: 'readout', label: '85 chunks · 1.2 GB · to LOD 3', v: '' }, { t: 'sub', label: 'Size cap · GB', items: ['No cap', '0.5 GB', '1 GB', '2 GB', '4 GB', '8 GB', '16 GB'].map((x) => ({ t: 'radio', label: x, key: 'prefs.atlasCap', value: x })) }, { t: 'live', label: 'Refine detail for the current view', act: 'toast', arg: 'Refining 12 tiles in view' }, { t: 'live', label: 'Refine viewshed for the current view', act: 'toast', arg: 'Viewshed refined for the view' }, { t: 'readout', label: 'Viewshed', v: 'never refined' }, { t: 'sep' }, { t: 'live', label: 'Export atlas…', act: 'filePick', arg: 'atlas destination' }, { t: 'live', label: 'Import atlas…', act: 'filePick', arg: 'atlas file' }, { t: 'sep' }, { t: 'live', label: 'Clear atlas cache now…', act: 'confirm', arg: 'Delete this world’s baked chunks? Project data is untouched.' }] },
      { t: 'sub', label: 'Chunk debug overlay', items: [{ t: 'check', label: 'Grid', key: 'prefs.dbgGrid' }, { t: 'check', label: 'Colors', key: 'prefs.dbgCol' }, { t: 'check', label: 'Labels', key: 'prefs.dbgLab' }, { t: 'sep' }, { t: 'check', label: 'Show tile borders on the map', key: 'prefs.tileBorders' }] },
      { t: 'band', label: 'MEMORY' },
      { t: 'sub', label: 'Undo history', items: [64, 128, 256, 512, 1024].map((x) => ({ t: 'radio', label: x + ' MB', key: 'prefs.undoMB', value: x })).concat([{ t: 'sep' }, { t: 'live', label: 'Clear undo history now', act: 'clearUndo' }]) },
      { t: 'readout', label: 'Working set', v: '1.4 GB of 8 GB' },
      { t: 'live', label: 'Clear caches…', act: 'confirm', arg: 'Delete this world’s baked chunks? Project data is untouched.' },
      { t: 'band', label: 'APPLICATION' },
      { t: 'live', label: 'Storage locations…', act: 'win:storage' },
      { t: 'live', label: 'Assistant…', act: 'win:assistantSettings' },
      { t: 'live', label: 'Quick bar…', act: 'win:quickbar' },
      { t: 'check', label: 'Open panels in their own window', key: 'prefs.popout', desktopOnly: true },
      { t: 'sub', label: 'Theme', items: [{ t: 'radio', label: 'Dark', key: 'prefs.theme', value: 'dark' }, { t: 'radio', label: 'Light', key: 'prefs.theme', value: 'light' }, { t: 'radio', label: 'Follow system', key: 'prefs.theme', value: 'system' }] },
      { t: 'sub', label: 'Units', items: [{ t: 'radio', label: 'Kilometres', key: 'prefs.units', value: 'km' }, { t: 'radio', label: 'Miles', key: 'prefs.units', value: 'mi' }, { t: 'radio', label: 'Nautical miles', key: 'prefs.units', value: 'nmi' }] },
      { t: 'sub', label: 'Handedness', items: [{ t: 'radio', label: 'Left', key: 'prefs.hand', value: 'left' }, { t: 'radio', label: 'Right', key: 'prefs.hand', value: 'right' }] },
      { t: 'live', label: 'Keyboard shortcuts…', act: 'win:shortcuts' },
      { t: 'band', label: 'TOUCH · NEW', mobile: true },
      { t: 'live', label: 'Touch & gestures…', act: 'win:touch', mobile: true }
    ] },
    { id: 'window', label: 'Window', d: IC.window, sub: 'regions · session · 3D · workspace · layouts', items: [
      { t: 'check', label: 'Left dock', key: 'win.left' }, { t: 'check', label: 'Context panel', key: 'win.right' },
      { t: 'sub', label: 'Context panel', items: [{ t: 'live', label: 'Inspect', accel: 'Ctrl+1', act: 'ctx', arg: 'inspect' }, { t: 'live', label: 'Note', accel: 'Ctrl+2', act: 'ctx', arg: 'note' }, { t: 'live', label: 'Ask (assistant)', accel: 'Ctrl+3', act: 'ctx', arg: 'ask' }, { t: 'sep' }, { t: 'live', label: 'Pin to the current subject', act: 'ctxPin' }, { t: 'live', label: 'Follow the selection', act: 'ctxFollow' }].concat(ctxModuleMenuItems()) }, { t: 'check', label: 'Timeline', key: 'win.timeline' }, { t: 'check', label: 'Status bar', key: 'win.status' }, { t: 'check', label: 'Domain rail', key: 'win.rail' },
      { t: 'sep' },
      { t: 'check', label: 'Session mode', accel: 'Ctrl+Shift+M', key: 'prefs.session' },
      { t: 'check', label: 'Diagnostics overlay', accel: 'Shift+D', key: 'prefs.diag' },
      { t: 'check', label: '3D view', key: 'dim3' },
      { t: 'sub', label: '3D capability', items: [{ t: 'readout', label: '3D tier', v: 'Basic' }, { t: 'live', label: 'Re-test 3D', act: 'toast', arg: '3D re-tested · Basic' }, { t: 'sub', label: 'Force tier', items: ['Automatic (as measured)', 'Off', 'Basic', 'Standard', 'Full'].map((x) => ({ t: 'radio', label: x, key: 'prefs.tier3d', value: x })) }] },
      { t: 'sep' },
      { t: 'sub', label: 'Workspace', items: [{ t: 'radio', label: 'World', key: 'dom', value: 'world', sm: true }, { t: 'radio', label: 'Civilization', key: 'dom', value: 'civil' }, { t: 'radio', label: 'Cartography', key: 'dom', value: 'carto' }] },
      { t: 'sub', label: 'Open windows', dyn: 'openWindows', items: [{ t: 'readout', label: 'No windows open', v: '' }, { t: 'check', label: 'Context panel', key: 'win.right' }] },
      { t: 'sep' },
      { t: 'live', label: 'Reset layout', act: 'resetLayout' },
      { t: 'sub', label: 'Layouts', items: [{ t: 'live', label: 'Default (reset layout)', act: 'resetLayout' }, { t: 'live', label: 'Generate', act: 'layout', arg: 'world/pipeline' }, { t: 'live', label: 'Sculpt', act: 'layout', arg: 'world/sculpt' }, { t: 'live', label: 'Cartography', act: 'layout', arg: 'carto/style' }, { t: 'live', label: 'Journey', act: 'layout', arg: 'story/journeys' }, { t: 'sep' }, { t: 'live', label: 'Save layout as…', act: 'toast', arg: 'Layout saved as “Layout 1”' }, { t: 'live', label: 'Forget layout…', act: 'toast', arg: 'Pick a stored layout to forget' }] }
    ] },
    { id: 'help', label: 'Help', d: IC.help, sub: 'documentation · shortcuts · credits · about', items: [
      { t: 'todo', label: 'Documentation', reason: 'An installed build has no README beside the project; the desktop checkout opens the documents folder.' },
      { t: 'live', label: 'Keyboard reference…', act: 'win:shortcuts', arg: 'ref' },
      { t: 'live', label: 'Gesture reference…', act: 'win:touch', mobile: true },
      { t: 'live', label: 'Credits & academic principles', act: 'win:credits' },
      { t: 'live', label: 'Generation info…', act: 'win:geninfo' },
      { t: 'live', label: 'Save diagnostic report…', act: 'toast', arg: 'Diagnostic report reviewed · saved to Exports' },
      { t: 'live', label: 'About', act: 'win:about' }
    ] }
  ];
}
/* Below 1 180 dp a touch tablet's bar is ☰ · File · World · Records · Story (Ruling DY, PQ6 follow-up). World there is
   the WORLD domain's own rows; Records is the desktop's Records menu; the ☰ holds the rest. */
/* ☰'s submenus are built with full=false (menus.gd::_overflow): a row ☰ promotes to its own top, and Edit's two Reset
   rows (which the narrow World carries), appear once. Separators left leading, trailing or doubled are dropped too. */
const PROMOTED = ['Undo', 'Redo', 'Undo history…', '⧉ Asset library', 'Landmark types', 'Units', 'Keyboard reference…', 'About', 'Reset generation parameters', 'Reset one stage'];   // Preferences keeps Theme and its editable Keyboard shortcuts… (_preferences(p, false, true))
function notPromoted(items) {
  const kept = items.filter((x) => x && !(x.t !== 'sep' && x.t !== 'band' && PROMOTED.includes(x.label)));
  const out = []; for (const x of kept) { if (x.t === 'sep' && (!out.length || out[out.length - 1].t === 'sep')) continue; out.push(x); }
  while (out.length && out[out.length - 1].t === 'sep') out.pop();
  return out;
}
function tabletOverflow(menus) {
  const by = (id) => menus.find((m) => m.id === id);
  return { id: 'ham', label: '☰', sub: '', items: [
    { t: 'live', label: 'Undo', accel: 'Ctrl+Z', act: 'undo', dyn: 'undo' }, { t: 'live', label: 'Redo', accel: 'Ctrl+Shift+Z', act: 'redo', dyn: 'redo' }, { t: 'live', label: 'Undo history…', act: 'history' },
    { t: 'sep' }, { t: 'live', label: '⧉ Asset library', accel: 'Shift+A', act: 'win:assets' }, by('assets').items.find((x) => x.label === 'Landmark types'),
    { t: 'sep' }, { t: 'live', label: 'Toggle theme', act: 'theme' }, by('prefs').items.find((x) => x.label === 'Units'),
    { t: 'sep' }, { t: 'live', label: 'Keyboard reference…', act: 'win:shortcuts', arg: 'ref' }, { t: 'live', label: 'About', act: 'win:about' },
    { t: 'sep' }, { t: 'sub', label: 'Edit', items: notPromoted(by('edit').items) }, { t: 'sub', label: 'Assets', items: notPromoted(by('assets').items) }, { t: 'sub', label: 'Preferences', items: notPromoted(by('prefs').items) }, { t: 'sub', label: 'Window', items: by('window').items }, { t: 'sub', label: 'Help', items: notPromoted(by('help').items) }
  ] };
}
/* The narrow bar's World: only the WORLD domain's own rows, which is what "World" means everywhere else (Ruling DY).
   "Generate world" is one label on every form factor (Ruling DX, PQ11). */
function tabletWorldMenu(menus) {
  return { id: 'world', label: 'World', d: IC.world, sub: 'generate · reset · pipeline · sculpt', items: [
    { t: 'live', label: 'Generate world', accel: 'Ctrl+R', act: 'run', sm: true }, { t: 'sep' },
    { t: 'live', label: 'Reset generation parameters', act: 'resetParams', sm: true }, menus.find((m) => m.id === 'edit').items.find((x) => x.label === 'Reset one stage'), { t: 'sep' },
    { t: 'radio', label: 'Generation pipeline', key: 'worldnode', value: 'pipeline', sm: true }, { t: 'radio', label: 'Sculpt', key: 'worldnode', value: 'sculpt', sm: true }, { t: 'sep' },
    { t: 'live', label: 'Seed & size…', act: 'newWorld', sm: true }
  ] };
}
/* The narrow bar's File: the desktop's File without Generate world, which that bar's World already holds (one row per
   command on a bar; a labelled judgement of Ruling DY's four-menu answer). */
function tabletFileMenu(menus) {
  const f = menus.find((m) => m.id === 'file');
  return Object.assign({}, f, { items: f.items.filter((x) => !(x && x.label === 'Generate world')) });
}
