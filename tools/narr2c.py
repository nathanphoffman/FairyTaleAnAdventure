#!/usr/bin/env python3
"""Converts the story text and tables in faery-tale-amiga/narr.asm into csrc/narr_data.c.
Usage (from the repo root): tools/narr2c.py > csrc/narr_data.c

Only `dc.b` / `dc.l` data lines are read. Each label becomes a C byte array holding exactly the
bytes the assembler would have produced (strings are NUL separated, XY escapes are byte 128).
"""
import re, sys

SRCS = ['faery-tale-amiga/narr.asm', 'faery-tale-amiga/fsubs.asm']
NAMES = {'ETX': 0, 'XY': 128}

def split_operands(text):
    """Splits a dc.b operand list on commas that are outside quotes; drops the comment."""
    items, cur, i, quote = [], '', 0, None
    while i < len(text):
        ch = text[i]
        if quote:
            cur += ch
            if ch == quote:
                quote = None
        elif ch in '\'"':
            quote = ch
            cur += ch
        elif ch == ';':
            break
        elif ch == ',':
            items.append(cur.strip()); cur = ''
        else:
            cur += ch
        i += 1
    if quote:
        # unterminated string, e.g.  "Tame the golden beast ... '   (the source has two of these)
        body = cur.strip()
        q = body[0]
        inner = body[1:]
        if inner.endswith("'"):
            inner = inner[:-1]
        cur = '\0UNTERM' + q + inner
    if cur.strip() or items:
        items.append(cur if cur.startswith('\0UNTERM') else cur.strip())
    return [x for x in items if x != '']

def operand_bytes(tok):
    if tok.startswith('\0UNTERM'):
        return list((tok[7] + tok[8:]).encode('latin-1')) if tok[7] == '"' else list(tok[8:].encode('latin-1'))
    if tok[0] in '\'"':
        return list(tok[1:-1].encode('latin-1'))
    if tok in NAMES:
        return [NAMES[tok]]
    if re.fullmatch(r'\$[0-9A-Fa-f]{5,8}', tok):          # a dc.l value: 4 bytes, big-endian
        return list(int(tok[1:], 16).to_bytes(4, 'big'))
    if re.fullmatch(r'[\d\$\(\)\+\-\*/ A-Fa-f]+', tok):
        expr = re.sub(r'\$([0-9A-Fa-f]+)', r'0x\1', tok).replace('/', '//')
        expr = re.sub(r'(?<![0-9A-Za-z])0+(?=\d)', '', expr)   # 02 -> 2 (decimal, not octal)
        return [eval(expr) & 0xff]
    raise ValueError('cannot read operand: ' + tok)

def c_string(data):
    out, line = [], ''
    for b in data:
        ch = chr(b)
        if ch == '"': piece = '\\"'
        elif ch == '\\': piece = '\\\\'
        elif 32 <= b < 127: piece = ch
        else: piece = '\\%03o' % b
        line += piece
        if len(line) > 90:
            out.append(line); line = ''
    out.append(line)
    return '\n\t'.join('"%s"' % l for l in out)

def main():
    keep = lambda n: n in ('_event_msg', '_place_tbl', '_inside_tbl', '_place_msg', '_inside_msg', '_speeches', '_titletext', '_hinor', '_hivar') or re.fullmatch(r'(msg\d+[ab]?|q\d|mst|qq)', n)
    blocks, order = {}, []
    tables = {}
    label = None
    lines = [l for src in SRCS for l in open(src, encoding='latin-1')]
    for raw in lines:
        line = raw.rstrip('\n')
        m = re.match(r'^(\w+)\s*(.*)$', line)
        rest = line
        if m and not line.startswith((' ', '\t', ';')):
            label = m.group(1)
            rest = m.group(2)
            if label in ('XY', 'ETX'):
                continue
            blocks.setdefault(label, [])
            if label not in order:
                order.append(label)
        rest = rest.strip()
        mb = re.match(r'dc\.b\s+(.*)', rest)
        ml = re.match(r'dc\.l\s+(.*)', rest)
        if mb and label and keep(label):
            for tok in split_operands(mb.group(1)):
                blocks[label] += operand_bytes(tok)
        elif ml and label and keep(label):
            toks = split_operands(ml.group(1))
            if all(t.startswith('$') for t in toks):        # long words of data (big-endian bytes)
                for t in toks:
                    blocks[label] += operand_bytes(t)
            else:                                           # offset table: name-base, ...
                tables.setdefault(label, []).extend(t.split('-')[0] for t in toks)
    print('/* narr_data.c - story text and tables, generated from narr.asm by tools/narr2c.py. Do not edit. */\n')
    for label in order:
        if label in ('mst', 'qq') or not keep(label) or not blocks[label]:
            continue
        name = label.lstrip('_')
        data = blocks[label]
        if name in ('place_tbl', 'inside_tbl'):
            print('unsigned char %s[] = {' % name)
            for i in range(0, len(data), 3):
                print('\t' + ', '.join(str(x) for x in data[i:i+3]) + ',')
            print('};\n')
        elif name in ('hinor', 'hivar'):
            data = data + [0] * (256 - len(data))
            print('unsigned char %s[256] = {' % name)
            for i in range(0, len(data), 16):
                print('\t' + ', '.join(str(x) for x in data[i:i+16]) + ',')
            print('};\n')
        elif name in ('event_msg', 'place_msg', 'inside_msg', 'speeches', 'titletext'):
            print('char %s[] =\n\t%s;\n' % (name, c_string(data)))
        else:
            print('static const char %s[] =\n\t%s;\n' % (name, c_string(data)))
    for tname, cname in (('mst', 'placard_msgs'), ('qq', 'question_msgs')):
        print('const char *const %s[] = {' % cname)
        for n in tables[tname]:
            print('\t%s,' % n)
        print('};\n')

main()
