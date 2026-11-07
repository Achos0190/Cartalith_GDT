"""Assemble Shell.dc.html from the template and the script parts (plain file reads/writes, no shell quoting)."""
import sys, os
root = sys.argv[1]
b = os.path.join(root, 'build')
parts = ['../engine/world.js', '../engine/render.js', 'd_params.js', 'd_menus.js', 'd_domains.js', 'd_place.js', 'd_rows.js', 'd_cats.js', 'd_screens.js', 'd_wins.js', 'd_ring.js', 'd_notes.js', 'd_vault.js', 'd_desk.js', 'd_vals.js', 'c_class.js']
js = '\n'.join(open(os.path.join(b, p), encoding='utf-8').read() for p in parts)
js = js.replace("if (typeof module !== 'undefined') module.exports = { genWorld, biomeOf, makeNoise, fbm, mulberry32, BIOME_COL };", '')
assert '</script' not in js.lower(), 'a closing script tag inside the JS would end the block early'
tpl = open(os.path.join(b, 'shell.tpl.html'), encoding='utf-8').read()
assert tpl.count('/*__SCRIPT__*/') == 1
out = tpl.replace('/*__SCRIPT__*/', js)
open(os.path.join(root, 'canvas', 'project', 'Shell.dc.html'), 'w', encoding='utf-8').write(out)
open(os.path.join(root, 'build', '_shell_check.js'), 'w', encoding='utf-8').write('(function(){\n' + js.replace('class Component extends DCLogic', 'class Component extends Object') + '\n})();\n')
print('Shell.dc.html', len(out), 'bytes; script', len(js))
