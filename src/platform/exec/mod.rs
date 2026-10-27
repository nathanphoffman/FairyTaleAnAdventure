//! Memory, libraries, tasks and device I/O. Mostly stand-ins: there is no multitasking OS to
//! talk to, so these hand back dummy objects that the game can hold and release.

mod device;

mod library;

mod memory;

pub use self::device::{
    begin_io, check_io, close_device, create_ext_io, create_port, create_std_io, delete_ext_io,
    delete_port, delete_std_io, do_io, open_device, send_io, wait_io,
};
pub use self::library::{close_library, find_task, open_library};
pub use self::memory::{alloc_mem, avail_mem, free_mem, type_of_mem};
