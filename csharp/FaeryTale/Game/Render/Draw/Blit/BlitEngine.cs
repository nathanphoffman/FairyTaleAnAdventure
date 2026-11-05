namespace FaeryTale.Game.Render;

/// The blitter in software: combines up to three source channels into a destination, with shifts
/// and a minterm, the way the original chip did.
///
/// The blitter combines up to three source channels (A, B, C) into a destination D, 16 bits at a
/// time, with an optional barrel shift of A and B, first/last word masks on A, and a "minterm"
/// saying how the bits combine. Words are big-endian in memory (leftmost pixels in the first byte).
internal static class BlitEngine
{
    // A word is 16 bits: two words are joined to shift across them. The minterm has 8 result bits,
    // picked by the A, B and C bits (4, 2, 1).
    private const int WORD_BITS = 16;
    private const int MINTERM_BITS = 8;
    private const int MINTERM_A = 4;
    private const int MINTERM_B = 2;
    private const int MINTERM_C = 1;

    /// Ascending blit (all the game's blits go left to right, top to bottom).
    ///
    /// For each line it reads a word from each channel in use, shifts A and B (the shifted-out
    /// bits of the previous word carry into the next, which lets a sprite sit at any pixel
    /// position, not just on 16-pixel boundaries), combines them with the minterm and writes the
    /// result to D.
    public static void Run(Blit blit, BlitMemory memory)
    {
        for (int line = 0; line < blit.Height; line++)
        {
            BlitRow(blit, memory);
            SkipRowModulos(blit);
        }
    }

    /// One 16 bit word of the minterm: for every bit, the result is picked by the A, B and C bits.
    ///
    /// The minterm is a table of 8 result bits, one for each combination of the A, B and C bits
    /// (bit number = A*4 + B*2 + C). For example 0xCA means "if A then B else C", which is how a
    /// mask (A) selects between sprite pixels (B) and the screen behind (C).
    public static uint MintermWord(uint minterm, uint a, uint b, uint c)
    {
        uint result = 0;

        for (int index = 0; index < MINTERM_BITS; index++)
        {
            // Bit index of the minterm is the result for that combination of the A, B and C bits.
            if (((minterm >> index) & 1) != 0)
            {
                result |= Pick(a, (index & MINTERM_A) != 0) & Pick(b, (index & MINTERM_B) != 0) & Pick(c, (index & MINTERM_C) != 0);
            }
        }

        return result & Blit.ALL_BITS;
    }

    private static uint Pick(uint word, bool set) => set ? word : ~word;

    // Blits one row of Width words, working through the A, B and C sources and the D destination
    // that are in use. A and B can be shifted, and carry bits over from the previous word.
    private static void BlitRow(Blit blit, BlitMemory memory)
    {
        uint previousA = 0;
        uint previousB = 0;

        for (int x = 0; x < blit.Width; x++)
        {
            uint aWord = NextAWord(blit, memory, x);
            uint bWord = 0;
            uint cWord = 0;

            if ((blit.Uses & Blit.USE_B) != 0)
            {
                bWord = memory.ReadWord(blit.B);
                blit.B = blit.B.Add(2);
            }

            if ((blit.Uses & Blit.USE_C) != 0)
            {
                cWord = memory.ReadWord(blit.C);
            }

            // Shifting right across two words: the bits that fell off the previous word enter from the left.
            uint shiftedA = ((previousA << WORD_BITS | aWord) >> blit.AShift) & Blit.ALL_BITS;
            uint shiftedB = ((previousB << WORD_BITS | bWord) >> blit.BShift) & Blit.ALL_BITS;

            previousA = aWord;
            previousB = bWord;

            uint result = MintermWord(blit.Minterm, shiftedA, shiftedB, cWord);

            if ((blit.Uses & Blit.USE_D) != 0)
            {
                memory.WriteWord(blit.D, result);
                blit.D = blit.D.Add(2);
            }

            if ((blit.Uses & Blit.USE_C) != 0)
            {
                blit.C = blit.C.Add(2);
            }
        }
    }

    // The A source word for column x of a row (all ones when A is not in use): the first and last
    // words of the row are masked.
    private static uint NextAWord(Blit blit, BlitMemory memory, int x)
    {
        if ((blit.Uses & Blit.USE_A) == 0)
        {
            return Blit.ALL_BITS;
        }

        uint word = memory.ReadWord(blit.A);

        blit.A = blit.A.Add(2);

        if (x == 0)
        {
            word &= blit.FirstWordMask;
        }

        if (x == blit.Width - 1)
        {
            word &= blit.LastWordMask;
        }

        return word;
    }

    // After a row, moves each place in use on by its modulo (bytes to skip to reach the next row).
    private static void SkipRowModulos(Blit blit)
    {
        if ((blit.Uses & Blit.USE_A) != 0)
        {
            blit.A = blit.A.Add(blit.AMod);
        }

        if ((blit.Uses & Blit.USE_B) != 0)
        {
            blit.B = blit.B.Add(blit.BMod);
        }

        if ((blit.Uses & Blit.USE_C) != 0)
        {
            blit.C = blit.C.Add(blit.CMod);
        }

        if ((blit.Uses & Blit.USE_D) != 0)
        {
            blit.D = blit.D.Add(blit.DMod);
        }
    }
}
