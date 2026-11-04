use std::mem::ManuallyDrop;
use std::net::*;
use std::num::NonZeroU16;
use std::os::fd::FromRawFd;
use std::path::*;

pub struct Settings {
    pub open:   bool,
    pub webdav: bool,
    pub addrs:  Vec<IpAddr>,
    pub port:   Option<NonZeroU16>,
    pub cache:  crate::fs::dir::Cache,
    pub root:   std::path::PathBuf,
    pub listen: Vec<ManuallyDrop<TcpListener>>,
}

impl Settings {
    /// ## Safety
    /// - This assumes it can take ownership of fds 3 .. ${LISTEN_FDS}+3 if `--systemd` was passed
    pub unsafe fn from_env_or_die() -> Self {
        let mut errors = false;
        let mut help = false;
        let mut open = false;
        let mut systemd = false;
        let webdav = true;
        let mut addrs = Vec::new();
        let mut port = Option::<NonZeroU16>::None;
        let mut root = Option::<PathBuf>::None;
        let mut listen = Vec::new();

        macro_rules! error   { ($($tt:tt)*) => {{ eprintln!($($tt)*); errors = true; }} }
        macro_rules! warning { ($($tt:tt)*) => {{ eprintln!($($tt)*); }} }

        let mut args = std::env::args_os();
        let _exe = args.next();
        while let Some(arg) = args.next() {
            match &*arg.to_string_lossy() {
                "--help"            => help = true,
                "--open"            => open = true,
                "--no-open"         => open = false,
                "--allow-all-ipv4" => addrs.push(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
                "--allow-all-ipv6" => addrs.push(IpAddr::V6(Ipv6Addr::UNSPECIFIED)),
                "--port" => {
                    let Some(n) = args.next() else { error!("error: expected port number after --port"); continue };
                    let Some(n) = n.to_str().and_then(|s| s.parse::<NonZeroU16>().ok()) else { error!("error: expected a port number after --port, got {n:?} instead"); continue };
                    if let Some(_prev) = port.replace(n) { warning!("warning: multiple --port [n] flags specified, only the last will apply") }
                },
                "--systemd" => systemd = true,
                flag if flag.starts_with("--") => error!("unrecognized flag {flag:?}"),

                _positional_lossy if root.is_none() => {
                    let path = PathBuf::from(arg);
                    if !path.is_dir() { error!("error: `{}` is not a directory", path.display()) }
                    root = Some(path);
                },
                positional => error!("error: expected at most one positional argument - the root directory - but recieved a second, {positional:?}"),
            }
        }

        if systemd {
            match unsafe { parse_systemd_env_vars() } { Ok(l) => listen = l, Err(()) => errors = true }
        } else if std::env::var_os("LISTEN_PID").is_some() {
            warning!("warning: --systemd not specified but ${{LISTEN_PID}} was set");
        }

        if help { println!("{}", include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/docs/usage.txt"))) }
        if errors { std::process::exit(1) }
        if help { std::process::exit(0) } // already printed help text

        Self {
            open,
            webdav,
            cache: crate::fs::dir::Cache::new(), // XXX: split off into a "context" type instead of hijacking settings?
            root: root.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_err| PathBuf::from("."))),

            // as a safer default:
            //  1.  only allow connections over a loopback address
            //  2.  pick a nonstandard loopback to avoid conflicting with other things on localhost
            //  3.  pick a nonstandard loopback to avoid exposing something via any reverse proxy forwarding
            //
            // default loopbacks:
            //  ::1         (ipv6)
            //  127.0.0.x   (linux)
            //  127.x.y.z   (windows / https://www.rfc-editor.org/rfc/rfc1122 )
            addrs: {
                if addrs.is_empty() && listen.is_empty() { addrs.push(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 99))) }
                addrs
            },
            port,
            listen,
        }
    }
}

/// ## Safety
/// - This assumes it can take ownership of fds 3 .. ${LISTEN_FDS}+3 if `--systemd` was passed
unsafe fn parse_systemd_env_vars() -> Result<Vec<ManuallyDrop<TcpListener>>, ()> {
    let listen_pid = std::env::var_os("LISTEN_PID").ok_or_else(|| eprintln!("error: specified --systemd, but ${{LISTEN_PID}} was not set"))?;
    let listen_pid = listen_pid.to_str().ok_or_else(|| eprintln!("error: could not parse ${{LISTEN_PID}} as a u32: invalid UTF-8"))?;
    let listen_pid = listen_pid.parse::<u32>().map_err(|err| eprintln!("error: could not parse ${{LISTEN_PID}} as a u32: {err}"))?;
    let actual_pid = std::process::id();
    if listen_pid != actual_pid { return Err(eprintln!("error: ${{LISTEN_PID}} ({listen_pid}) != std::process::id() ({actual_pid})")) }

    let fds = std::env::var_os("LISTEN_FDS").ok_or_else(|| eprintln!("error: specified --systemd, but ${{LISTEN_FDS}} was not set"))?;
    let fds = fds.to_str().ok_or_else(|| eprintln!("error: could not parse ${{LISTEN_FDS}} as a usize: invalid UTF-8"))?;
    let fds = fds.parse::<usize>().map_err(|err| eprintln!("error: could not parse ${{LISTEN_FDS}} as a usize: {err}"))?;
    if fds == 0 { return Err(eprintln!("error: ${{LISTEN_FDS}} cannot be 0")) }

    //dbg!(std::env::var_os("LISTEN_FDNAMES")); // Some("mmuhttpd.socket[:...]")

    Ok((3..).take(fds).map(|fd| ManuallyDrop::new(unsafe { TcpListener::from_raw_fd(fd) })).collect::<Vec<_>>())
}
