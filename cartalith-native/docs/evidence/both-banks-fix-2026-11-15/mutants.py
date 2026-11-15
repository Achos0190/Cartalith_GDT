import subprocess, sys, os, hashlib
ROOT='C:/cw/bbfix/cartalith-native/crates/cartalith-urban/src/'
M=[
('riverbank.rs','pub const VILLAGE_LOW_SHARE: f64 = 0.8;','pub const VILLAGE_LOW_SHARE: f64 = 0.5;'),
('riverbank.rs','pub const VILLAGE_AIM_SHARE: f64 = 1.1;','pub const VILLAGE_AIM_SHARE: f64 = 0.6;'),
('riverbank.rs','pub const VILLAGE_FITS: usize = 4;','pub const VILLAGE_FITS: usize = 1;'),
('riverbank.rs','pub const VILLAGE_MIN_SCALE: f64 = 0.1;','pub const VILLAGE_MIN_SCALE: f64 = 0.9;'),
('riverbank.rs','return Some(RiverbankPass::FarBank { share: share.min(FAR_BANK_MAX_RATIO) });','return Some(RiverbankPass::FarBank { share });'),
('riverbank.rs','if t.pop_target < BOTH_BANKS_MIN_POP && t.pop > t.pop_target * RIVER_HOUSING_CAP {','if t.pop_target < BOTH_BANKS_MIN_POP && t.pop > t.pop_target {'),
('riverbank.rs','lay(least_over.0, true)','lay(least_over.0, false)'),
('riverbank.rs','if t.pop > hi && t.pop < least_over.1 {','if t.pop > hi {'),
('riverbank.rs','Some(if site.bank_side(e1) == site.bank_side(market) { e2 } else { e1 })','Some(if site.bank_side(e1) == site.bank_side(market) { e1 } else { e2 })'),
('growth.rs','while len < budget && far_len < cap && tries < FAR_BANK_TRIES {','while len < budget && tries < FAR_BANK_TRIES {'),
('growth.rs','if site.bank_side(o) == mkt_side || !crate::riverbank::far_bank_buildable(site, o) {','if !crate::riverbank::far_bank_buildable(site, o) {'),
('growth.rs','pub const FAR_BANK_EXPLORE: f64 = 0.35;','pub const FAR_BANK_EXPLORE: f64 = 0.0;'),
('growth.rs','pub const FAR_BANK_TRIES: i32 = 2600;','pub const FAR_BANK_TRIES: i32 = 10;'),
('growth.rs','        if d > opts.max_rf {\n            continue;\n        }\n        if !explore && r.u() > 1.35','        if d > 50.0 {\n            continue;\n        }\n        if !explore && r.u() > 1.35'),
('generate.rs','        if let RiverbankPass::FarBank { share } = pass\n            && both_banks\n','        if let RiverbankPass::FarBank { share } = pass\n            && false\n'),
('generate.rs','matches!(pass, RiverbankPass::Village { trim: true, .. })','matches!(pass, RiverbankPass::Village { .. })'),
('wallhug.rs','pub const CLOSE_R_M: f64 = 40.0;','pub const CLOSE_R_M: f64 = 10.0;'),
('wallhug.rs','pub const OUTLINE_OPEN_CELLS: u32 = 2;','pub const OUTLINE_OPEN_CELLS: u32 = 0;'),
('wallhug.rs','pub const OUTLINE_MINOR_SHARE: f64 = 0.05;','pub const OUTLINE_MINOR_SHARE: f64 = 0.0;'),
('wallhug.rs','pub const UNTANGLE_MAX_SHARE: f64 = 0.02;','pub const UNTANGLE_MAX_SHARE: f64 = 0.6;'),
('wallhug.rs','pub const BANK_EXEMPT_M: f64 = 10.0;','pub const BANK_EXEMPT_M: f64 = 0.0;'),
('wallhug.rs','    if (far as f64) < (MIN_LOTS as f64).max(near as f64 * FAR_BANK_LOT_SHARE) {','    if (far.min(near) as f64) < (MIN_LOTS as f64).max(near as f64 * FAR_BANK_LOT_SHARE) {'),
('wallhug.rs','let plaza = plaza.filter(|p| rb && p.len() >= 3);','let plaza = plaza.filter(|p| false && p.len() >= 3);'),
('wallhug.rs','let built = BuiltMass { hull, spans_water: spans, exact: rb };','let built = BuiltMass { hull, spans_water: spans, exact: false };'),
('wallhug.rs','        if comp[j * nx + i] != best {\n            return None;','        if false {\n            return None;'),
('wallhug.rs','        if poly_area(&cut).abs() > UNTANGLE_MAX_SHARE * area {','        if false {'),
('wallhug.rs','        if one_bank_by_lots(&sides, mkt_side).is_none() {\n            spans_water = true;','        if false {\n            spans_water = true;'),
('wallhug.rs','if faces(1.0) && faces(-1.0) {','if faces(1.0) || faces(-1.0) {'),
('wallhug.rs','if area_ratio > MAX_AREA_RATIO && !(spans && !wall_state.spans_water) {','if area_ratio > MAX_AREA_RATIO {'),
('wallhug.rs','let ring = if rb && ring_self_crosses(ring) {','let ring = if false {'),
('fortify.rs','let base = if bmh.exact { hull.clone() } else { convex_hull(&hull) };','let base = if false { hull.clone() } else { convex_hull(&hull) };'),
('wallhug.rs','    if rb && rk_site && !spans_water {\n        let mut every','    if false {\n        let mut every'),
('wallhug.rs','margin_pts = if rb && rk { inland.clone() } else { mass.clone() };','margin_pts = mass.clone();'),
('plaza.rs','&& let Some((d, m)) = riverbank_square(site, anchors, g, l, wd)','&& let Some((d, m)) = None::<(Vec2, Vec2)>'),
('plaza.rs','pub const SQUARE_RIVER_CLEAR_M: f64 = 15.0;','pub const SQUARE_RIVER_CLEAR_M: f64 = 0.0;'),
('plaza.rs','let ok = |p: Vec2| !site.is_water(p) && site.bank_side(p) == mkt_side && site.river_dist(p) >= clear;','let ok = |p: Vec2| !site.is_water(p) && site.river_dist(p) >= clear;'),
('amenities.rs','pub const MARKET_WALL_CLEAR_M: f64 = 30.0;','pub const MARKET_WALL_CLEAR_M: f64 = 0.0;'),
('amenities.rs','            if within.is_some_and(|f| !f(p)) {','            if false {'),
('generate.rs','ring.is_some().then_some(&inside as &dyn Fn(Vec2) -> bool),','None,'),
('wallhug.rs','    } else if rb && rk_site && spans_water {','    } else if false {'),
('wallhug.rs','            if plaza.is_some_and(|pz| !point_in_poly(poly_centroid(pz), ring)) {','            if false {'),
('wallhug.rs','        if comp[j * nx + i] != best {','        if false {'),
('plaza.rs','&& pop >= PLAZA_MARKET_POP','&& pop >= 0.0'),
('generate.rs','.filter(|p| p.kind == crate::plaza::PlazaKind::Market)','.filter(|_| true)'),
]
sel = [int(x) for x in sys.argv[1:]] if len(sys.argv)>1 else range(len(M))
env=dict(os.environ, CARGO_TARGET_DIR='C:/cw/bbfix_mut_target')
for i in sel:
    f,old,new=M[i]
    p=ROOT+f
    src=open(p,encoding='utf-8').read()
    h=hashlib.sha256(src.encode()).hexdigest()
    assert src.count(old)==1,(i,f,old,src.count(old))
    try:
        open(p,'w',encoding='utf-8').write(src.replace(old,new))
        r=subprocess.run(['cargo','test','-p','cartalith-urban','--lib','--','--test-threads','8'],cwd='C:/cw/bbfix/cartalith-native',env=env,capture_output=True,text=True,timeout=3000)
        out=r.stdout+r.stderr
        if 'could not compile' in out or 'error[' in out:
            v='BUILD_ERROR'
        elif r.returncode!=0:
            v='KILLED'
        else:
            v='SURVIVED'
        fails=[l for l in out.split('\n') if l.endswith('FAILED') and l.startswith('test ')]
        print(f'{v} #{i} {f}: {old.strip()[:90]!r} -> {new.strip()[:90]!r} {fails[:4]}',flush=True)
    finally:
        open(p,'w',encoding='utf-8').write(src)
        assert hashlib.sha256(open(p,encoding='utf-8').read().encode()).hexdigest()==h
print('DONE')
