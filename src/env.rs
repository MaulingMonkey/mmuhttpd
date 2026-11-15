use std::env::var_os;
use std::ffi::{OsStr, OsString};
use std::os::fd::BorrowedFd;
use std::sync::OnceLock;

use core::ffi::c_int as int;



/// `${LISTEN_PID}` : [`u32`]
///
/// -   <code>Ok(None)</code> if `${LISTEN_PID}` was not set
/// -   <code>Ok(Some(...))</code> if `${LISTEN_PID}` was a valid [`u32`]
/// -   <code>Err(())</code> if `${LISTEN_PID}` was not a valid [`u32`]
///
pub fn listen_pid() -> Result<Option<u32>, ()> {
    static PID : OnceLock<Result<Option<u32>, ()>> = OnceLock::new();
    *PID.get_or_init(||{
        let Some(listen_pid) = var_os("LISTEN_PID") else { return Ok(None) };
        let listen_pid = listen_pid.to_str().ok_or_else(|| eprintln!("error: could not parse ${{LISTEN_PID}} as a u32: invalid UTF-8"))?;
        let listen_pid = listen_pid.parse::<u32>().map_err(|err| eprintln!("error: could not parse ${{LISTEN_PID}} as a u32: {err}"))?;
        Ok(Some(listen_pid))
    })
}



/// `${LISTEN_FDS}` : [`u16`]
///
/// -   <code>Ok(None)</code> if `${LISTEN_FDS}` was not set
/// -   <code>Ok(Some(...))</code> if `${LISTEN_FDS}` was a valid [`u16`]
/// -   <code>Err(())</code> if `${LISTEN_FDS}` was not a valid [`u16`]
///
pub fn listen_fds_count() -> Result<Option<u16>, ()> {
    static FDS : OnceLock<Result<Option<u16>, ()>> = OnceLock::new();
    *FDS.get_or_init(||{
        let fds = var_os("LISTEN_FDS").ok_or_else(|| eprintln!("error: specified --systemd, but ${{LISTEN_FDS}} was not set"))?;
        let fds = fds.to_str().ok_or_else(|| eprintln!("error: could not parse ${{LISTEN_FDS}} as a u16: invalid UTF-8"))?;
        let fds = fds.parse::<u16>().map_err(|err| eprintln!("error: could not parse ${{LISTEN_FDS}} as a u64: {err}"))?;
        Ok(Some(fds))
    })
}



#[derive(Clone, Debug)]
pub struct FdAndNames<'n> {
    pub fd:     BorrowedFd<'n>,
    pub name:   Option<&'n str>,
}

impl FdAndNames<'static> {
    /// Self { fd, native: "unknown", name: "unknown" }
    #[allow(dead_code)]
    pub unsafe fn from_unknown_fd(fd: int) -> Self { Self { fd: unsafe { BorrowedFd::borrow_raw(fd) }, name: None } }

    /// 3 .. 3+`${LISTEN_FDS}` x `${LISTEN_FDNAMES}`
    pub unsafe fn from_env() -> impl Iterator<Item = Self> {
        let start = int::from(3);
        let n   = listen_fds_count().unwrap_or(None).map(int::from).unwrap_or(0);
        let fds = start .. (start + n);

        #[cfg(not(unix))] return fds.map(Self::from_unknown_fd);
        #[cfg(    unix )] return {
            use std::os::unix::ffi::OsStrExt;

            fn names_os() -> &'static OsStr {
                static NAMES : OnceLock<OsString> = OnceLock::new();
                NAMES.get_or_init(|| var_os("LISTEN_FDNAMES").unwrap_or_default())
            }

            fn names_lossy() -> &'static str {
                static NAMES_LOSSY : OnceLock<String> = OnceLock::new();
                NAMES_LOSSY.get_or_init(|| names_os().to_string_lossy().into_owned())
            }

            let names_os    = names_os().as_bytes() .split(|b| *b == b':').chain(core::iter::repeat(b"unknown".as_slice())).map(<OsStr as OsStrExt>::from_bytes);
            let names_lossy = names_lossy()         .split(           ":").chain(core::iter::repeat("unknown"));

            fds.zip(names_os).zip(names_lossy).map(|((fd, _native), name)| FdAndNames { fd: unsafe { BorrowedFd::borrow_raw(fd) }, name: Some(name) })
        };
    }
}
