use crate::env::FdAndNames;

use std::io::{self, ErrorKind};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::os::fd::{AsRawFd, FromRawFd};

use core::mem::ManuallyDrop;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};



pub struct TcpListener2 {
    owned:          bool,
    accepting:      AtomicBool,
    listener:       ManuallyDrop<std::net::TcpListener>,
}

impl core::ops::Drop for TcpListener2 {
    fn drop(&mut self) {
        if self.owned {
            self.owned = false;
            unsafe { ManuallyDrop::drop(&mut self.listener) };
        }
    }
}

impl TcpListener2 {
    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        if self.accepting.load(Ordering::Relaxed) {
            self.listener.accept()
        } else {
            Err(io::Error::new(io::ErrorKind::Interrupted, "TcpListener2::shutdown called"))
        }
    }

    pub fn bind(addrs: impl ToSocketAddrs) -> io::Result<TcpListener2> {
        TcpListener::bind(addrs).map(Self::from)
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    #[allow(dead_code)]
    pub fn shutdown(&self) -> io::Result<()> {
        if !self.accepting.load(Ordering::Relaxed) { return Ok(()) } // already shut down
        let local_addr = self.listener.local_addr()?;
        self.accepting.store(false, Ordering::Relaxed);
        let timeout = std::time::Duration::from_millis(100);
        loop {
            match TcpStream::connect_timeout(&local_addr, timeout) {
                Ok(_stream) => return Ok(()), // fini
                Err(err) => match err.kind() {
                    ErrorKind::TimedOut => continue, // TODO: ???
                    _kind               => return Err(err),
                }
            }
        }
    }
}

impl From<TcpListener> for TcpListener2 {
    fn from(listener: TcpListener) -> Self {
        Self {
            owned:      true,
            accepting:  AtomicBool::new(true),
            listener:   ManuallyDrop::new(listener),
        }
    }
}

impl TryFrom<FdAndNames<'_>> for TcpListener2 {
    type Error = io::Error;

    fn try_from(FdAndNames { fd, name, .. }: FdAndNames<'_>) -> Result<Self, Self::Error> {
        use crate::libc::*;

        let raw_fd = fd.as_raw_fd();

        macro_rules! err { ( $($tt:tt)* ) => { Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{}{}", format_args!("TcpListener2::try_from({raw_fd} {name:?}): ", raw_fd=raw_fd, name=name), format_args!($($tt)*)))) } }

        let mut ty : core::ffi::c_int = 0;
        let mut ty_len = core::mem::size_of_val(&ty) as _;
        match unsafe { getsockopt(raw_fd, SOL_SOCKET, SO_TYPE, NonNull::new(&mut ty).map(|p| p.cast()), &mut ty_len) } {
            0 => {
                assert_eq!(ty_len as usize, core::mem::size_of_val(&ty));
                if ty != SOCK_STREAM { return err!("expected a socket of type SOCK_STREAM ({SOCK_STREAM}, TCP), not {actual}", actual = sock_ty_display(ty)) }
            },
            -1  => return err!("getsockopt(..., SOL_SOCKET, SO_TYPE, ...) returned -1, might not be a socket?"), // ideally we'd check errno for EBADF or ENOTSOCK
            ret => panic!("bug: getsockopt({raw_fd} {name:?}, ...) returned {ret}, expected 0 (success) or -1 (see errno)"),
        }

        let mut accept = 0_u64;
        let mut accept_len = core::mem::size_of_val(&accept) as _;
        match unsafe { getsockopt(raw_fd, SOL_SOCKET, SO_ACCEPTCONN, NonNull::new(&mut accept).map(|p| p.cast()), &mut accept_len) } {
            0 => {
                assert!(accept_len as usize <= core::mem::size_of_val(&accept));
                if accept == 0 { return err!("expected a socket with SO_ACCEPTCONN"); }
            },
            -1  => return err!("getsockopt(..., SOL_SOCKET, SO_ACCEPTCONN, ...) returned -1, might not be a socket?"), // ideally we'd check errno for EBADF or ENOTSOCK
            ret => panic!("bug: getsockopt({raw_fd} {name:?}, ...) returned {ret}, expected 0 (success) or -1 (see errno)"),
        }

        match fd.try_clone_to_owned() {
            Ok(owned) => Ok(Self {
                owned:      true,
                accepting:  AtomicBool::new(true),
                listener:   ManuallyDrop::new(owned.into()),
            }),
            Err(err) => match err.kind() {
                ErrorKind::Unsupported  => Ok(Self {
                    owned:      false,
                    accepting:  AtomicBool::new(true),
                    listener:   ManuallyDrop::new(unsafe { TcpListener::from_raw_fd(raw_fd) }),
                }),
                _kind => Err(err),
            }
        }
    }
}

fn sock_ty_display(ty: core::ffi::c_int) -> impl core::fmt::Display {
    use crate::libc::*;
    match ty {
        SOCK_STREAM     => format!("SOCK_STREAM ({ty}, TDP)"),
        SOCK_DGRAM      => format!("SOCK_DGRAM ({ty}, UDP)"),
        SOCK_SEQPACKET  => format!("SOCK_SEQPACKET ({ty}, TCP?)"),
        SOCK_RAW        => format!("SOCK_RAW ({ty}, ???)"),
        _other          => format!("SOCK_??? ({ty}, ???)"),
    }
}
