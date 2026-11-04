#![allow(dead_code, non_camel_case_types)]
#![cfg(target_os = "linux")]
//! This all should really be replaced with [the `libc` crate](https://docs.rs/libc/), but I said 0 dependencies in the readme and I'm stubborn.

use core::ptr::NonNull;

type int    = core::ffi::c_int;
type void   = core::ffi::c_void;

// NOTE:  Doublecheck/enforce all types and values are as expected in libc.cpp!

pub type fd = int;

/// \[[man.archlinux.org](https://man.archlinux.org/man/core/man-pages/socklen_t.3type.en)\]
pub type socklen_t = int;

pub const SO_TYPE           : int =  3;
pub const SO_ACCEPTCONN     : int = 30;

pub const SOL_SOCKET        : int =  1;

// (level=SOL_SOCKET, optname=SO_TYPE)
pub const SOCK_STREAM       : int = 1;
pub const SOCK_DGRAM        : int = 2;
pub const SOCK_RAW          : int = 3;
pub const SOCK_SEQPACKET    : int = 5;

extern "C" {
    /// \[[man.archlinux.org](https://man.archlinux.org/man/getsockopt.2.en)\]
    pub fn getsockopt(
        sockfd:     fd,
        level:      int,
        optname:    int,
        optval:     Option<NonNull<void>>,
        optlen:     &mut socklen_t,
    ) -> int;
}
