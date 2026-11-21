import csv,sys,statistics as S
def stats(f):
    r=list(csv.DictReader(open(f)))
    q=lambda a,x:sorted(a)[int(round((len(a)-1)*x))]
    out={}
    for v in ('off','proto'):
        T=[x for x in r if x['variant']==v and x['walled']=='1']
        sh=[int(x['wall_sharp_convex'])+int(x['wall_sharp_reflex']) for x in T]
        rf=[int(x['wall_sharp_reflex']) for x in T]
        lh=[float(x['wall_len_over_hull']) for x in T]
        w=[int(x['wall_sharpest_deg']) for x in T]
        out[v]=(len(T),S.median(sh),q(sh,.9),max(sh),sum(sh),sum(rf),round(S.median(lh),3),round(q(lh,.9),3),max(lh))
        print(v,'n',len(T),'sharp med/p90/max/total',S.median(sh),q(sh,.9),max(sh),sum(sh),'reflex tot',sum(rf),'| len/hull med/p90/max',out[v][6:])
    off={(x['seed'],x['town']):x for x in r if x['variant']=='off' and x['walled']=='1'}
    on={(x['seed'],x['town']):x for x in r if x['variant']=='proto' and x['walled']=='1'}
    worse=[k for k in on if k in off and int(on[k]['wall_sharp_convex'])+int(on[k]['wall_sharp_reflex'])>int(off[k]['wall_sharp_convex'])+int(off[k]['wall_sharp_reflex'])]
    big=[k for k in on if k in off and float(on[k]['wall_len_over_hull'])>float(off[k]['wall_len_over_hull'])+0.05]
    print('towns with more sharp vertices',len(worse),'; len/hull up >0.05',len(big),[ (k,off[k]['wall_len_over_hull'],on[k]['wall_len_over_hull']) for k in big])
stats(sys.argv[1])
