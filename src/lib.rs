#[cfg(target_os = "linux")]
mod linux {
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;

    extern crate glob;
    use self::glob::glob;

    extern crate nix;
    use self::nix::sys::stat::stat;

    // Returns the dev_t corresponding to the current process's controlling tty
    pub fn get_ctty_dev() -> Result<u64, Box<Error>> {
        // /proc/self/stat contains the ctty's device id in field 7
        // Open it and read its contents to a string
        let mut stat_f = File::open("/proc/self/stat")?;
        let mut stat = String::new();
        stat_f.read_to_string(&mut stat)?;

        // Start looking at the string two positions after the last ')'
        // This is because the data inside the () may contain spaces
        let mut start_idx = stat.rfind(')').unwrap_or(0);
        if start_idx == 0 {
            return Err(From::from("Failed to parse /proc/self/stat!"));
        }
        start_idx += 2;
        
        // Split by whitespace into array to easily access indices
        let values_str = &stat[start_idx..];
        let mut values = values_str.split_whitespace();

        // Extract 5th field from start (represented as i32)
        let dev = values.nth(4).ok_or_else(|| {
            let err: Box<Error> = From::from("Failed to parse /proc/self/stat!");
            err
        })?;
        let dev_int = dev.parse::<i32>()?;
        
        // Cast result to u64 and return
        Ok(dev_int as u64)
    }

    // Returns a full path to a tty or pseudo tty that corresponds with the given dev_t
    pub fn get_path_for_dev(dev: u64) -> Result<String, Box<Error>> {
        // Check all devices in /dev/pts/* and /dev/tty* for a match 
        let patterns = ["/dev/pts/*", "/dev/tty"];

        for i in 0..patterns.len() {
            for entry in glob(patterns[i])? {
                let path = match entry {
                    Ok(p) => p,
                    Err(_) => { // Silently continue
                        continue;
                    }
                };

                // See if this device matches the request
                let stat = match stat(&path) {
                    Ok(s) => s,
                    Err(_) => { // Silently continue
                        continue;
                    }
                };

                if dev == stat.st_rdev {
                    // Found device, return it
                    return Ok(String::from(path.to_str().unwrap()));
                }
            }
        }

        Err(From::from("Failed to look up path for given device!"))
    }
}
#[cfg(target_os = "linux")]
pub use linux::*;

/// For FreeBSD and macOS, it's probably not worth it to recreate the kinfo_proc struct
/// in Rust and use FFI bindings to call sysctl, so I'm instead using a small C wrapper.
#[cfg(any(target_os = "freebsd", target_os = "macos"))]
mod bsd {
    use std::error::Error;
    use std::ffi::CStr;

    extern crate libc;
    use self::libc::{S_IFCHR, c_int, mode_t, dev_t, c_char};

    extern "C" {
        // Provided by wrapper (see bsd.c)
        fn _get_ctty_dev() -> u64;

        // Provided by system libc
        fn devname_r(dev: dev_t, type_: mode_t, buf: *mut u8, len: c_int) -> *mut c_char;
    }
    

    pub fn get_ctty_dev() -> Result<u64, Box<Error>> {
        let res = unsafe { _get_ctty_dev() };
        if res == 0 {
            return Err(From::from("Failed to determine controlling tty!"));
        }
        Ok(res)
    }

    pub fn get_path_for_dev(dev: u64) -> Result<String, Box<Error>> {
        let mut buf: Vec<u8> = Vec::with_capacity(255);
        unsafe {
            let res: *mut c_char = devname_r(dev as dev_t, S_IFCHR, buf.as_mut_ptr(), 255);
            // On failure, result will be NULL, &'?', or &'#' depending on OS
            if res.is_null() || *res as u8 == b'?' || *res as u8 == b'#' {
                return Err(From::from("Failed to determine name for given device!"));
            }

            // Convert the buffer into an owned string
            let res_owned = CStr::from_ptr(res).to_string_lossy().into_owned();

            // Append /dev/ to the beginning and return it
            Ok(format!("{}{}", "/dev/", res_owned))
        }
    }


}
#[cfg(any(target_os = "freebsd", target_os = "macos"))]
pub use bsd::*;

#[cfg(test)]
mod tests {
    use ::get_path_for_dev;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_get_path_for_dev() {
        get_path_for_dev(30);
    }
}
