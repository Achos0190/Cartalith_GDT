"""Keep each recorded request (the bytes the app sent) and only the usage line of its response: python compact.py <src dir> <dst dir>"""
import glob, json, os, sys
os.makedirs(sys.argv[2], exist_ok=True)
for f in glob.glob(os.path.join(sys.argv[1], 'proxy_*.jsonl')):
    out = []
    for l in open(f, encoding='utf8'):
        r = json.loads(l); u = [x for x in r['response'].splitlines() if x.startswith('data: {') and '"usage"' in x]
        r['response'] = u[-1] if u else ''; out.append(json.dumps(r))
    open(os.path.join(sys.argv[2], os.path.basename(f)), 'w', encoding='utf8').write('\n'.join(out) + '\n')
