# Project instructions for Claude Code

- **Do not make git commits** (and do not push). The user makes all commits. When a piece of work is
  finished and the tests pass, leave the changes in the working tree, list what changed in a few
  lines, and suggest a one-line commit message. Only commit if the user explicitly asks in that message.
- The C to Rust port (`csrc/` to `rsrc/`) is described in `modern_engine_plan.md`, section 4a. Read its
  "HANDOFF" and "Roadmap" parts before porting anything.
- `csrc/` is the reference and is not edited, except the `#ifndef RSRC_*` guards and the `main()` cuts
  already described in the plan.
- Check a port with `cargo build` (this links the C), `cargo test`, `tests/regress.sh` and
  `tests/state_regress.sh` (run the last two in the background; every scenario must say `ok`).
