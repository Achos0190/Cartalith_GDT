import re, statistics, sys
def load(f):
    d={}; k=-1
    lines=open(f).read().split('\n')
    # rows precede their seed summary line; assign block by counting summaries
    blk=0
    for line in lines:
        m=re.match(r'\s+#\s*(\d+) \S+ river_w (\d+) street ([\d.]+) built-to-bank (\d+) m',line)
        if m: d[(blk,int(m.group(1)))]=(float(m.group(4)),float(m.group(3)))
        if line.startswith('seed '): blk+=1
    return d
for gw in ['384','2048']:
    a=load(f'audit_before_{gw}.txt'); b=load(f'audit_after_{gw}.txt')
    keys=[k for k in a if k in b]
    ga=[a[k][0] for k in keys]; gb=[b[k][0] for k in keys]
    over_a=sum(1 for k in keys if a[k][0]>a[k][1]); over_b=sum(1 for k in keys if b[k][0]>b[k][1])
    closer=sum(1 for k in keys if b[k][0]<a[k][0])
    print(gw,'paired',len(keys),'median before',statistics.median(ga),'after',statistics.median(gb),'house over street before',over_a,'after',over_b,'closer after',closer)
