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

/* The game's own random number generator (fsubs.asm). Must match exactly, and must replace the
 * libc rand() (renamed with a macro in amiga.h). */
extern long seed1;   /* defined in fmain.c (long is 32 bit there, see amiga.h) */

int fta_rand(void)
{
	/* mulu.w multiplies only the low 16 bits of the seed */
	seed1 = (long)((unsigned long)((unsigned short)seed1) * 45821UL) + 1;
	unsigned int r = ((unsigned int)seed1 >> 6) | ((unsigned int)seed1 << 26); /* ror.l #6 */
	return (int)(r & 0x7fffffff);
}
long bitrand(long mask) { return fta_rand() & mask; }
long rand2(void) { return fta_rand() & 1; }
long rand4(void) { return fta_rand() & 3; }
long rand8(void) { return fta_rand() & 7; }
long rand64(void) { return fta_rand() & 63; }
long rand256(void) { return fta_rand() & 255; }
long rnd(long n) { return (long)((unsigned long)((unsigned short)fta_rand()) % (unsigned long)(n & 0xffff)); }
