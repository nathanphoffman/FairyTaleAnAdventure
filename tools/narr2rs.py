#!/usr/bin/env python3
"""Converts the story text and tables in faery-tale-amiga/narr.asm into src/game/data/story_text.rs.
Usage (from the repo root): tools/narr2rs.py > src/game/data/story_text.rs

Only `dc.b` / `dc.l` data lines are read. Each label becomes a C byte array holding exactly the
bytes the assembler would have produced (strings are NUL separated, XY escapes are byte 128).
Text comes out as byte strings with one more NUL at the end, like a C string.
"""

import ast, operator, re, sys

SRCS = ['faery-tale-amiga/narr.asm', 'faery-tale-amiga/fsubs.asm']
ENCODING = 'latin-1'
RUST_NAMES = {'titletext': 'title_text'}
UNTERMINATED_MARK = '\0UNTERM'
NAMES = {'ETX': 0, 'XY': 128}

ARITHMETIC = {ast.Add: operator.add, ast.Sub: operator.sub, ast.Mult: operator.mul,
              ast.FloorDiv: operator.floordiv}

def evaluate(expr):
    """Works out a whole-number expression made of + - * // and parentheses (no eval needed)."""

    def walk(node):
        if isinstance(node, ast.Constant) and isinstance(node.value, int):
            return node.value

        if isinstance(node, ast.BinOp) and type(node.op) in ARITHMETIC:
            return ARITHMETIC[type(node.op)](walk(node.left), walk(node.right))

        if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
            return -walk(node.operand)

        raise ValueError('cannot work out: ' + expr)

    return walk(ast.parse(expr.strip(), mode='eval').body)

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

        cur = UNTERMINATED_MARK + q + inner

    if cur.strip() or items:
        items.append(cur if cur.startswith(UNTERMINATED_MARK) else cur.strip())

    return [x for x in items if x != '']

def operand_bytes(tok):
    if tok.startswith(UNTERMINATED_MARK):
        return list((tok[7] + tok[8:]).encode(ENCODING)) if tok[7] == '"' else list(tok[8:].encode(ENCODING))

    if tok[0] in '\'"':
        return list(tok[1:-1].encode(ENCODING))

    if tok in NAMES:
        return [NAMES[tok]]

    if re.fullmatch(r'\$[0-9A-Fa-f]{5,8}', tok):          # a dc.l value: 4 bytes, big-endian
        return list(int(tok[1:], 16).to_bytes(4, 'big'))

    if re.fullmatch(r'[\d\$\(\)\+\-\*/ A-Fa-f]+', tok):
        expr = re.sub(r'\$([0-9A-Fa-f]+)', r'0x\1', tok).replace('/', '//')
        expr = re.sub(r'(?<![0-9A-Za-z])0+(?=\d)', '', expr)   # 02 -> 2 (decimal, not octal)

        return [evaluate(expr) & 0xff]

    raise ValueError('cannot read operand: ' + tok)

def rust_bytes(data):
    """A Rust byte string holding `data` plus a final NUL (the end of the last string). Long text is
    wrapped with a backslash at the end of the line, which skips the next line's indent; a space at
    the start of a wrapped line is written as \\x20 so that it is kept."""
    out, line = [], ''

    for b in list(data) + [0]:
        ch = chr(b)

        if ch == '"': piece = '\\"'
        elif ch == '\\': piece = '\\\\'
        elif ch == ' ' and not line and out: piece = '\\x20'
        elif 32 <= b < 127: piece = ch
        else: piece = '\\x%02x' % b

        line += piece

        if len(line) > 90:
            out.append(line); line = ''

    out.append(line)

    return 'b"' + '\\\n        '.join(out) + '"'

def rust_array(data, per_line):
    rows = [', '.join(str(x) for x in data[i:i+per_line]) for i in range(0, len(data), per_line)]

    return '[\n    ' + ',\n    '.join(rows) + ',\n]'

def main():
    keep = lambda n: n in ('_event_msg', '_place_tbl', '_inside_tbl', '_place_msg', '_inside_msg', '_speeches', '_titletext', '_hinor', '_hivar') or re.fullmatch(r'(msg\d+[ab]?|q\d|mst|qq)', n)
    blocks, order = {}, []
    tables = {}
    label = None
    lines = [l for src in SRCS for l in open(src, encoding=ENCODING)]

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

    print('//! Story text and tables, generated from narr.asm by tools/narr2rs.py. Do not edit.')
    print('//!')
    print('//! Messages are NUL separated, with one more NUL at the end. A byte of 128 in a placard or')
    print('//! the title is a position escape (followed by x / 2 and y).\n')
    placards = []

    for label in order:
        if label in ('mst', 'qq') or not keep(label) or not blocks[label]:
            continue

        name = label.lstrip('_')
        data = blocks[label]

        if name in ('place_tbl', 'inside_tbl'):
            print('pub static %s: [u8; %d] = %s;\n' % (name.upper(), len(data), rust_array(data, 3)))
        elif name in ('hinor', 'hivar'):
            data = data + [0] * (256 - len(data))
            print('pub static %s: [u8; 256] = %s;\n' % (name.upper(), rust_array(data, 16)))
        elif name in ('event_msg', 'place_msg', 'inside_msg', 'speeches', 'titletext'):
            print('pub static %s: &[u8] =\n    %s;\n' % (RUST_NAMES.get(name, name).upper(), rust_bytes(data)))
        elif name.startswith('msg'):
            print('const %s: &[u8] =\n    %s;\n' % (name.upper(), rust_bytes(data)))

    print('/// The story pages, in order.')
    print('pub static PLACARD_MSGS: [&[u8]; %d] = [' % len(tables['mst']))

    for n in tables['mst']:
        print('    %s,' % n.upper())

    print('];')

main()
