/* asm_port.c - routines that were 68000 assembly, rewritten in C.
 * Each one notes which asm routine it replaces. Data is big-endian on disk, so anything read
 * straight from a file into an integer needs swapping on this little-endian machine.
 */
#include "ftale.h"

/* ---- from fsubs.asm: IFF helpers (globals live in iffsubs.c) ---- */
extern long file_length, myfile, header, blocklength;
extern long bytecount;
extern unsigned char *packdata;
extern char compress;

static unsigned long be32(unsigned long v)
{
	return (v >> 24) | ((v >> 8) & 0xff00) | ((v << 8) & 0xff0000) | (v << 24);
}

/* reads a 4 byte chunk id; compared against the id string in memory, so no swap */
void ReadHeader(void)
{
	Read(myfile, &header, 4);
	file_length -= 4;
}

/* reads a big-endian chunk length */
void ReadLength(void)
{
	Read(myfile, &blocklength, 4);
	blocklength = (long)be32((unsigned long)blocklength);
	file_length -= blocklength + 4;
}

/* unpacks one scan line of an IFF body (ByteRun1, or plain copy) into dest */
void unpack_line(unsigned char *dest)
{
	long j, upc;
	unsigned char b;

	if (!compress)
	{	long i;
		for (i = 0; i < bytecount; i++) *dest++ = *packdata++;
		return;
	}
	j = 0;
	while (j < bytecount)
	{	upc = (signed char)*packdata++;
		if (upc >= 0)
		{	for (; upc >= 0; upc--) { *dest++ = *packdata++; j++; }
		}
		else
		{	upc = -upc;
			b = *packdata++;
			for (; upc >= 0; upc--) { *dest++ = b; j++; }
		}
	}
}
