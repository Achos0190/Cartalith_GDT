import sys
P='C:/cw/f12_proj/godot-project/shell/'
f=sys.argv[2]; old=sys.argv[3].replace('<TAB>',chr(9)); new=sys.argv[4].replace('<TAB>',chr(9))
s=open(P+f,encoding='utf8',newline='').read()
if sys.argv[1]=='apply':
    assert s.count(old)==1
    open(P+f+'.orig','w',encoding='utf8',newline='').write(s)
    open(P+f,'w',encoding='utf8',newline='').write(s.replace(old,new))
else:
    open(P+f,'w',encoding='utf8',newline='').write(open(P+f+'.orig',encoding='utf8',newline='').read())
    import os; os.remove(P+f+'.orig')
