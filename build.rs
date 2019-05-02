fn main() {
    // xclip and xsel paths are inserted at compile time on Linux
    #[cfg(all(
        feature = "clipboard",
        any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
        )
    ))]
    {
        println!("cargo:rerun-if-env-changed=XCLIP_PATH");
        println!("cargo:rerun-if-env-changed=XSEL_PATH");
    }
}
