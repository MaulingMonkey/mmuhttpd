use std::net::*;
use std::io::Write;
use std::path::*;

pub struct Settings {
    pub open:   bool,
    pub webdav: bool,
    pub bind:   IpAddr,
    pub cache:  crate::fs::dir::Cache,
    pub root:   std::path::PathBuf,
}

impl Settings {
    pub fn from_env_or_die() -> Self {
        let mut help = false;
        let mut open = false;
        let webdav = true;
        let mut bind = Option::<IpAddr>::None;
        let mut root = Option::<PathBuf>::None;

        let mut args = std::env::args_os();
        let _exe = args.next();
        for arg in args {
            match &*arg.to_string_lossy() {
                "--help" => {
                    if !help {
                        help = true;
                        let mut stdout = std::io::stdout().lock();
                        for line in include_str!("help.txt").trim_end().split('\n').map(|l| l.trim_end()) {
                            let _ = writeln!(stdout, "{line}");
                        }
                    }
                },
                "--open"            => open = true,
                "--no-open"         => open = false,
                "--allow-all-ipv4" => {
                    if let Some(_prev) = bind.replace(IpAddr::V4(Ipv4Addr::UNSPECIFIED)) {
                        eprintln!("warning: multiple --allow-* flags specified, only the last will apply")
                    }
                },
                "--allow-all-ipv6" => {
                    if let Some(_prev) = bind.replace(IpAddr::V6(Ipv6Addr::UNSPECIFIED)) {
                        eprintln!("warning: multiple --allow-* flags specified, only the last will apply")
                    }
                },
                flag if flag.starts_with("--") => panic!("unrecognized flag {flag:?}"),

                _positional if root.is_none() => root = Some(PathBuf::from(arg)),
                positional => panic!("expected at most one positional argument - the root directory - but recieved a second, {positional:?}"),
            }
        }

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
            bind: bind.unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 99))),
        }
    }
}
