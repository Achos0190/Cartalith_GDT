"""Table of the FX sweep: python analyze.py <scratch dir with c8192/c12288/c16384>. Per size and scenario: success
(params = a card with >=1 valid change that Apply took; edit_ridge = a card with a valid op that Apply changed the field
for), request count, prompt tokens per request (the server's own usage), first-byte latency, generation tok/s
(completion tokens over total minus first byte), prefill tok/s (prompt tokens over first byte, requests >= 1 500 tokens
only), and the VRAM samples (max dedicated GB, max shared GB) over the size's whole run."""
import glob, json, os, statistics as st, sys
D = sys.argv[1]
def med(x): return round(st.median(x), 1) if x else None
for ctx in (8192, 12288, 16384):
    d = os.path.join(D, 'c%d' % ctx)
    if not os.path.isdir(d): continue
    out = {}
    for sc in ('params', 'edit_ridge'):
        runs = sorted(glob.glob(os.path.join(d, 'result_%s_r*.json' % sc)))
        ok = 0; n = 0; reqs = []; firsts = []; gens = []; pres = []; peak_prompt = []
        for rf in runs:
            res = json.load(open(rf, encoding='utf8')); n += 1
            ev = {e['what']: e for e in res['events']}
            if sc == 'params':
                good = bool(res.get('card_found')) and res.get('card_valid', 0) >= 1 and 'applied' in ev
            else:
                good = bool(res.get('card_found')) and bool(res.get('valid')) and ev.get('applied', {}).get('field_changed') is True
            ok += good
            pf = rf.replace('result_', 'proxy_').replace('.json', '.jsonl')
            rows = [json.loads(l) for l in open(pf, encoding='utf8')]
            reqs.append(len(rows)); mp = 0
            for r in rows:
                u = {}
                for ln in r['response'].splitlines():
                    if ln.startswith('data: {') and '"usage"' in ln:
                        try: u = json.loads(ln[6:]).get('usage') or u
                        except Exception: pass
                p, c = u.get('prompt_tokens'), u.get('completion_tokens')
                if p: mp = max(mp, p)
                firsts.append(r['first_byte_s'])
                if c and c >= 30 and r['total_s'] - r['first_byte_s'] > 0.2: gens.append(c / (r['total_s'] - r['first_byte_s']))
                if p and p >= 1500 and r['first_byte_s'] > 0.5: pres.append(p / r['first_byte_s'])
            peak_prompt.append(mp)
            if not good: print('  FAIL', rf, {k: res.get(k) for k in ('card_found', 'card_valid', 'valid', 'rejected', 'structural')}, res.get('panel_final', '')[-220:].replace('\n', ' '))
        out[sc] = (ok, n, med(reqs), med(peak_prompt), max(peak_prompt) if peak_prompt else None, med(firsts), round(max(firsts), 1) if firsts else None, med(gens), med(pres))
    v = [l.split() for l in open(os.path.join(d, 'vram.txt')) if len(l.split()) == 3] if os.path.exists(os.path.join(d, 'vram.txt')) else []
    ded = max((float(x[1]) for x in v), default=None); sh = max((float(x[2]) for x in v), default=None)
    print('ctx', ctx, 'vram max dedicated GB', ded, 'shared GB', sh)
    for sc, o in out.items():
        print('  %-10s ok %d/%d  reqs/run %s  peak prompt tok med %s max %s  first-byte s med %s max %s  gen tok/s med %s  prefill tok/s med %s' % ((sc,) + o))
