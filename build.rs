fn main() {
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
        // Select clipboard binary method
        #[cfg(not(feature = "clipboard-crate"))]
        println!("cargo:rustc-cfg=feature=\"clipboard-bin\"");

        // xclip and xsel paths are inserted at compile time
        println!("cargo:rerun-if-env-changed=XCLIP_PATH");
        println!("cargo:rerun-if-env-changed=XSEL_PATH");
    }

    #[cfg(all(
        feature = "clipboard",
        not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
        ))
    ))]
    {
        // Select clipboard crate method
        #[cfg(not(feature = "clipboard-bin"))]
        println!("cargo:rustc-cfg=feature=\"clipboard-crate\"");
    }
}
