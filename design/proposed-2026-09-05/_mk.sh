#!/bin/sh
f="$1"
{ printf '%s\n' '<!doctype html>' '<html>' '<head>' '  <meta charset="utf-8">' \
    '  <script src="./support.js"></script>' '</head>' '<body>' '<x-dc>' '<helmet>' '  <style>'
  cat _shared.css
  printf '%s\n' '  </style>' '</helmet>'
  cat "$f.body"
  printf '%s\n' '</x-dc>' '</body>' '</html>'
} > "$f.dc.html"
rm -f "$f.body"
wc -c "$f.dc.html"
