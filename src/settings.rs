use std::net::*;
use std::num::NonZeroU16;
use std::path::*;

pub struct Settings {
    pub open:   bool,
    pub webdav: bool,
    pub addr:   IpAddr,
    pub port:   Option<NonZeroU16>,
    pub cache:  crate::fs::dir::Cache,
    pub root:   std::path::PathBuf,
}

impl Settings {
    pub fn from_env_or_die() -> Self {
        let mut errors = false;
        let mut help = false;
        let mut open = false;
        let webdav = true;
        let mut addr = Option::<IpAddr>::None;
        let mut port = Option::<NonZeroU16>::None;
        let mut root = Option::<PathBuf>::None;

        macro_rules! error   { ($($tt:tt)*) => {{ eprintln!($($tt)*); errors = true; }} }
        macro_rules! warning { ($($tt:tt)*) => {{ eprintln!($($tt)*); }} }

        let mut args = std::env::args_os();
        let _exe = args.next();
        while let Some(arg) = args.next() {
            match &*arg.to_string_lossy() {
                "--help"            => help = true,
                "--open"            => open = true,
                "--no-open"         => open = false,
                "--allow-all-ipv4" => {
                    if let Some(_prev) = addr.replace(IpAddr::V4(Ipv4Addr::UNSPECIFIED)) {
                        warning!("warning: multiple --allow-* flags specified, only the last will apply");
                    }
                },
                "--allow-all-ipv6" => {
                    if let Some(_prev) = addr.replace(IpAddr::V6(Ipv6Addr::UNSPECIFIED)) {
                        warning!("warning: multiple --allow-* flags specified, only the last will apply");
                    }
                },
                "--port" => {
                    let Some(n) = args.next() else { error!("error: expected port number after --port"); continue };
                    let Some(n) = n.to_str().and_then(|s| s.parse::<NonZeroU16>().ok()) else { error!("error: expected a port number after --port, got {n:?} instead"); continue };
                    if let Some(_prev) = port.replace(n) { warning!("warning: multiple --port [n] flags specified, only the last will apply") }
                },
                flag if flag.starts_with("--") => error!("unrecognized flag {flag:?}"),

                _positional_lossy if root.is_none() => {
                    let path = PathBuf::from(arg);
                    if !path.is_dir() { error!("error: `{}` is not a directory", path.display()) }
                    root = Some(path);
                },
                positional => error!("error: expected at most one positional argument - the root directory - but recieved a second, {positional:?}"),
            }
        }

        if help { println!("{}", include_str!("help.txt")) }
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
            addr: addr.unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 99))),
            port,
        }
    }
}
