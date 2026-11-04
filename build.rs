use std::path::PathBuf;



fn main() {
    let target_os           =               std::env::var_os("CARGO_CFG_TARGET_OS"  ).expect("${CARGO_CFG_TARGET_OS}");

    let cargo_manifest_dir  = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR"   ).expect("${CARGO_MANIFEST_DIR}"));
    let out_dir             = PathBuf::from(std::env::var_os("OUT_DIR"              ).expect("${OUT_DIR}"));

    if target_os == "linux" {
        match std::process::Command::new("cc")
            .current_dir(&cargo_manifest_dir)
            .args("-c src/libc.cpp -o".split(' ')).arg(out_dir.join("libc.o"))
            .status()
        {
            Ok(exit) if exit.success()  => {},
            Ok(_exit)                   => panic!("compiling libc.cpp failed"),
            Err(err)                    => panic!("compiling libc.cpp failed: {err}"),
        }
    }
}
