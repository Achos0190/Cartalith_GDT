import subprocess,sys,os
R='C:/cw/rtown/cartalith-native/crates/'
env=dict(os.environ,CARGO_TARGET_DIR='C:/cw/rtown_target')
M=[
 ('M1 min_by->max_by in drawn_stretch',R+'cartalith-civ/src/urban_adapter.rs','.min_by(|a, b| a.1.1.total_cmp(&b.1.1))','.max_by(|a, b| a.1.1.total_cmp(&b.1.1))',['test','--release','-p','cartalith-civ','--lib','a_town_river_follows']),
 ('M2 drop DRAWN_BIND_CELLS filter',R+'cartalith-civ/src/urban_adapter.rs','.filter(|(_, (_, d))| *d <= DRAWN_BIND_CELLS)','',['test','--release','-p','cartalith-civ','--lib','a_town_river_follows']),
 ('M3 skip the river_path replacement',R+'cartalith-civ/src/urban_adapter.rs','Some(runs) => drawn_stretch(runs, traced[bi], (px, py), box_rad_g, &grid_to_local).unwrap_or(local),','Some(_) => local,',['test','--release','-p','cartalith-civ','--lib','a_town_river_follows']),
 ('M4 app default off',R+'cartalith-godot/src/params.rs','p.urban_river_smooth = true;','p.urban_river_smooth = false;',['test','--release','-p','cartalith-godot','--test','params_mapping','town_river_curve']),
 ('M5 parity default on',R+'cartalith-engine/src/lib.rs','urban_river_smooth: false,','urban_river_smooth: true,',['test','--release','-p','cartalith-godot','--test','params_mapping','town_river_curve']),
 ('M6 loads-when-absent on',R+'cartalith-godot/src/params.rs','("urban_river_smooth", Value::Bool(false)),','("urban_river_smooth", Value::Bool(true)),',['test','--release','-p','cartalith-godot','--test','params_mapping','town_river_curve']),
]
which=sys.argv[1:] 
for name,f,a,b,cmd in M:
    if which and name.split()[0] not in which: continue
    s=open(f,encoding='utf-8',newline='').read()
    assert s.count(a)==1,(name,s.count(a))
    open(f,'w',encoding='utf-8',newline='').write(s.replace(a,b))
    try:
        r=subprocess.run(['cargo']+cmd,cwd='C:/cw/rtown/cartalith-native',env=env,capture_output=True,text=True)
        out=r.stdout+r.stderr
        if 'error[' in out or 'could not compile' in out: print(name,'BUILD_ERROR')
        elif r.returncode!=0 and 'FAILED' in out: print(name,'KILLED')
        else: print(name,'SURVIVED')
    finally:
        open(f,'w',encoding='utf-8',newline='').write(s)
