# WCAG 2.x contrast for the v2.72 :root tokens (dark and light), with rgba composited over its real ground.
def hx(s):
    s=s.lstrip('#'); return tuple(int(s[i:i+2],16) for i in (0,2,4))
def over(fg,a,bg): return tuple(fg[i]*a+bg[i]*(1-a) for i in range(3))
def lum(c):
    def t(x):
        x/=255; return x/12.92 if x<=0.03928 else ((x+0.055)/1.055)**2.4
    r,g,b=c; return 0.2126*t(r)+0.7152*t(g)+0.0722*t(b)
def cr(a,b):
    la,lb=lum(a),lum(b); return (max(la,lb)+0.05)/(min(la,lb)+0.05)
D=dict(bg='#0d0e0f',panel='#121314',panel2='#17191a',ins='#101112',ink='#e8ebec',muted='#a9adb0',dim='#8d9296',faint='#6f7478',accent='#e0a34a',accent2='#6a9bc4',warn='#c96a5a',good='#6fae7d',stale='#b9a878',accInk='#141005')
Lt=dict(bg='#f4f2ee',panel='#f2f0ec',panel2='#fbfaf7',ins='#e7e5e0',ink='#1b1a17',muted='#4e4f4a',dim='#5b5c56',faint='#6e6f69',accent='#a4650f',accent2='#2f6a9b',warn='#a8442f',good='#3f7a4e',stale='#8a6f2e',accInk='#fbfaf7')
WASH={'dark':((224,163,74),.09,.16),'light':((164,101,15),.08,.15)}
LINE={'dark':((255,255,255),.10,.16),'light':((0,0,0),.12,.18)}
for name,T in (('dark',D),('light',Lt)):
    print(f'== {name} ==')
    grounds={k:hx(T[k]) for k in ('bg','panel','panel2','ins')}
    wc,w1,w2=WASH[name]
    grounds['wash/panel']=over(wc,w1,grounds['panel']); grounds['wash2/panel']=over(wc,w2,grounds['panel'])
    hdr='ink      '+' '.join(f'{g:>11}' for g in grounds)
    print(hdr)
    for ink in ('ink','muted','dim','faint','accent','accent2','warn','good','stale'):
        row=[]
        for g,gc in grounds.items():
            v=cr(hx(T[ink]),gc); row.append(f'{v:6.2f}{"*" if v<4.5 else " "}{"!" if v<3 else " "}   ')
        print(f'{ink:8} '+' '.join(row))
    print('accInk on accent', round(cr(hx(T['accInk']),hx(T['accent'])),2))
    print('#18191a on accent (.pill.on)', round(cr(hx('#18191a'),hx(T['accent'])),2))
    print('#1a1206 on accent (.lp-item.on)', round(cr(hx('#1a1206'),hx(T['accent'])),2))
    lc,l1,l2=LINE[name]
    for g in ('panel','panel2','ins','bg'):
        print(f'border(.16/.18) vs {g}: line {cr(over(lc,l1,grounds[g]),grounds[g]):.2f}  border {cr(over(lc,l2,grounds[g]),grounds[g]):.2f}')
    print('accent edge vs panel', round(cr(hx(T['accent']),grounds['panel']),2))
# hard-coded, theme-independent surfaces
pop=over((14,17,23),.97,hx('#0d0e0f'))
for c,n in (('#e8dcc4','popup text'),('#c8b998','popup row label'),('#b08d54','popup x'),('#d8b97a','popup heading')):
    print(f'{n} {c} on popup bg: {cr(hx(c),pop):.2f}')
