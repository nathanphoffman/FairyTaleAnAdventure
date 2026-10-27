#!/usr/bin/env python3
"""Converts a binary PPM (P6) screenshot to PNG, scaled 2x vertically for the 4:3 look.
Usage: tools/ppm2png.py in.ppm out.png"""

import sys, zlib, struct

def main(src, dst):
    data = open(src, 'rb').read()
    parts = data.split(b'\n', 3)
    w, h = map(int, parts[1].split())
    pix = parts[3]
    rows = []

    for y in range(h):
        row = b'\x00' + pix[y * w * 3:(y + 1) * w * 3]
        rows += [row, row]  # double each line

    raw = b''.join(rows)

    def chunk(tag, body):
        c = struct.pack('>I', len(body)) + tag + body

        return c + struct.pack('>I', zlib.crc32(tag + body) & 0xffffffff)

    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h * 2, 8, 2, 0, 0, 0))
    png += chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b'')
    open(dst, 'wb').write(png)

main(sys.argv[1], sys.argv[2])
