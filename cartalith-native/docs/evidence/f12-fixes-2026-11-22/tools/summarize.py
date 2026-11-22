import json, glob, os, sys
D = sys.argv[1]
rows = []
for pf in sorted(glob.glob(os.path.join(D, 'proxy_*.jsonl'))):
    tag = os.path.basename(pf)[6:-6]
    reqs = []
    for line in open(pf, encoding='utf8'):
        r = json.loads(line)
        q = json.loads(r['request'])
        calls = ''
        usage = {}
        for ln in r['response'].splitlines():
            if ln.startswith('data: {'):
                try:
                    d = json.loads(ln[6:])
                except Exception:
                    continue
                for ch in d.get('choices', []):
                    for tc in ch.get('delta', {}).get('tool_calls', []) or []:
                        f = tc.get('function', {})
                        calls += (f.get('name') or '') + (f.get('arguments') or '')
                if d.get('usage'):
                    usage = d['usage']
        args = calls.replace('propose_edit_actions', '', 1)
        ops = []
        try:
            for e in json.loads(args).get('edits', []):
                ops.append({k: v for k, v in e.items() if k not in ('region_id', 'reason')})
        except Exception:
            ops = ['<unparsed> ' + args[:120]]
        reqs.append({'bytes': len(r['request']), 'msgs': len(q['messages']), 'prompt': usage.get('prompt_tokens'), 'completion': usage.get('completion_tokens'),
                     'first_s': round((r.get('first_byte_s') if 'first_byte_s' in r else (r.get('first_byte_ms') or 0)/1000) or 0, 1), 'total_s': round(r['total_s'] if 'total_s' in r else r.get('total_ms',0)/1000, 1), 'ops': ops})
    res = {}
    rf = os.path.join(D, 'result_%s.json' % tag)
    if os.path.exists(rf):
        d = json.load(open(rf, encoding='utf8'))
        res = {'valid': [v['operation'] for v in d.get('valid', [])], 'rejected': [(r.get('rule'), r.get('why', '')[:70]) for r in d.get('rejected', [])] if d.get('rejected') else [], 'structural': d.get('structural', ''),
               'preview_changed': (d.get('preview') or {}).get('changed'), 'applied': [e.get('field_changed') for e in d['events'] if e['what'] == 'applied'], 'undo_restored': [e.get('field_restored') for e in d['events'] if e['what'] == 'undone'],
               'first_card_ops': d.get('first_card_ops')}
    print(tag)
    for q in reqs:
        print('   ', q)
    print('   ', res)
