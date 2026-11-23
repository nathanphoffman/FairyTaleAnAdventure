#!/usr/bin/env python3
"""Converts a binary PPM (P6) screenshot to PNG, scaled 2x vertically for the 4:3 look.
Usage: tools/ppm2png.py in.ppm out.png"""

import struct
import sys
import zlib

# A PPM file is three header lines (magic, size, max value) then the pixel bytes: 3 per pixel.
PPM_HEADER_LINES = 3
BYTES_PER_PIXEL = 3
CRC_MASK = 0xFFFFFFFF
PNG_BIT_DEPTH = 8


def main(src, dst):
    data = open(src, "rb").read()
    parts = data.split(b"\n", PPM_HEADER_LINES)
    w, h = map(int, parts[1].split())
    pix = parts[PPM_HEADER_LINES]
    rows = []

    for y in range(h):
        row = b"\x00" + pix[y * w * BYTES_PER_PIXEL : (y + 1) * w * BYTES_PER_PIXEL]
        rows += [row, row]  # double each line

    raw = b"".join(rows)

    def chunk(tag, body):
        c = struct.pack(">I", len(body)) + tag + body

        return c + struct.pack(">I", zlib.crc32(tag + body) & CRC_MASK)

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h * 2, PNG_BIT_DEPTH, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")
    open(dst, "wb").write(png)


main(sys.argv[1], sys.argv[2])
