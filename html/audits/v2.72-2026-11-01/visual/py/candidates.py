import sys
sys.path.insert(0,'/tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-visual/py')
from contrast import hx,cr,over
def hexs(c): return '#'+''.join(f'{round(x):02x}' for x in c)
def scale_to(base, ground, target, lighter=True):
    # move base toward white (dark theme) or black (light theme) along a straight line until it meets target
    b=hx(base); g=hx(ground)
    for i in range(0,1001):
        t=i/1000
        c=tuple(b[k]+( (255-b[k]) if lighter else -b[k])*t for k in range(3))
        c=tuple(round(x) for x in c)
        if cr(c,g)>=target: return hexs(c), round(cr(c,g),2)
print('dark faint ->4.5 on panel2 #17191a:', scale_to('#6f7478','#17191a',4.5,True))
print('dark faint ->4.5 on wash2/panel:', scale_to('#6f7478',hexs(over((224,163,74),.16,hx('#121314'))),4.5,True))
print('light faint ->4.5 on ins #e7e5e0:', scale_to('#6e6f69','#e7e5e0',4.5,False))
print('light accent ->4.5 on ins #e7e5e0:', scale_to('#a4650f','#e7e5e0',4.5,False))
print('light accent ->4.5 on wash2/panel:', scale_to('#a4650f',hexs(over((164,101,15),.15,hx('#f2f0ec'))),4.5,False))
for g in ('#f2f0ec','#fbfaf7','#e7e5e0','#f4f2ee'):
    print('design light --acc #9a5d0c on',g, round(cr(hx('#9a5d0c'),hx(g)),2), '| accInk #fbf7ef on it', round(cr(hx('#fbf7ef'),hx('#9a5d0c')),2))
print('design light --acc on design wash .12/pan:', round(cr(hx('#9a5d0c'),over((164,101,15),.12,hx('#fbfaf7'))),2))
print('dark dim on ins', round(cr(hx('#8d9296'),hx('#101112')),2))
print('light good #3f7a4e ->4.5 on ins:', scale_to('#3f7a4e','#e7e5e0',4.5,False))
print('light stale ->4.5 on ins:', scale_to('#8a6f2e','#e7e5e0',4.5,False))
# scale bar: #cfd4dc on rgba(10,12,16,.55) over a white snowcap and over mid sand
for mapc in ('#ffffff','#d9c79c','#5b8fb0'):
    g=over((10,12,16),.55,hx(mapc)); print('scalebar text on .55 scrim over',mapc, round(cr(hx('#cfd4dc'),g),2), '| at .85:', round(cr(hx('#cfd4dc'),over((10,12,16),.85,hx(mapc))),2))
for mapc in ('#ffffff','#d9c79c'):
    g=over((10,12,17),.74,hx(mapc)); print('legend dim text on .74 scrim over',mapc, round(cr(hx('#8d9296'),g),2))
    gl=over((251,250,247),.85,hx('#1b2a3a')); 
print('light legend dim #5b5c56 on .85 parchment over deep water #1b2a3a', round(cr(hx('#5b5c56'),over((251,250,247),.85,hx('#1b2a3a'))),2))
