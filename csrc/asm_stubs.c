/* asm_stubs.c - the few Amiga-only symbols the game still names.
 *
 * trapper         the 68000 trap handler hook; nothing to do on Linux
 * HandlerInterface the Amiga input handler; input.rs / input.c replace it
 * WBenchMsg       Workbench start message; main() is never started from Workbench
 */
long trapper(void) { return 0; }
long HandlerInterface(void) { return 0; }
void *WBenchMsg;
