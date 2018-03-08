#[cfg(any(target_os = "freebsd", target_os = "macos"))]
extern crate cc;

fn main() {
    // On FreeBSD and macOS, build the sysctl wrapper
    #[cfg(any(target_os = "freebsd", target_os = "macos"))]
    cc::Build::new()
        .file("src/bsd.c")
        .compile("bsdwrapper");
}
