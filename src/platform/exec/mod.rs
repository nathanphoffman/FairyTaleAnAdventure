//! Memory, libraries, tasks and device I/O. Mostly stand-ins: there is no multitasking OS to
//! talk to, so these hand back dummy objects that the game can hold and release.

mod device;
mod library;
mod memory;
