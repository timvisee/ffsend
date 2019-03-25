fn main() {
    // xclip and xsel paths are inserted at compile time on Linux
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rerun-if-env-changed=XCLIP_PATH");
        println!("cargo:rerun-if-env-changed=XSEL_PATH");
    }
}
