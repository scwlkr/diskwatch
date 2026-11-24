pub struct Disk {
    pub free: u64,
    pub total: u64,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod platform {
    use super::Disk;
    use std::ffi::{c_char, c_int, c_ulong};
    use std::mem::MaybeUninit;

    #[cfg(target_os = "macos")]
    type Count = u32;

    // On 64-bit Linux both glibc and musl use 64-bit block and inode counts.
    #[cfg(target_os = "linux")]
    type Count = u64;

    #[repr(C)]
    struct Statvfs {
        block_size: c_ulong,
        fragment_size: c_ulong,
        blocks: Count,
        free_blocks: Count,
        available_blocks: Count,
        files: Count,
        free_files: Count,
        available_files: Count,
        filesystem_id: c_ulong,
        flags: c_ulong,
        name_max: c_ulong,
        #[cfg(target_os = "linux")]
        spare: [c_int; 6],
    }

    extern "C" {
        fn statvfs(path: *const c_char, result: *mut Statvfs) -> c_int;
    }

    pub fn query() -> Result<Disk, &'static str> {
        #[cfg(target_os = "macos")]
        if let Ok(disk) = query_path(c"/System/Volumes/Data") {
            return Ok(disk);
        }

        query_path(c"/")
    }

    fn query_path(path: &std::ffi::CStr) -> Result<Disk, &'static str> {
        let mut stats = MaybeUninit::<Statvfs>::zeroed();
        // The C string is NUL-terminated and stats matches the target's statvfs ABI.
        if unsafe { statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
            return Err("could not query disk space");
        }
        let stats = unsafe { stats.assume_init() };
        let fragment_size = u64::from(stats.fragment_size);
        let free = u64::from(stats.available_blocks).checked_mul(fragment_size);
        let total = u64::from(stats.blocks).checked_mul(fragment_size);
        match (free, total) {
            (Some(free), Some(total)) => Ok(Disk { free, total }),
            _ => Err("disk capacity out of range"),
        }
    }
}

#[cfg(all(target_os = "linux", not(target_pointer_width = "64")))]
compile_error!("diskwatch requires a 64-bit Linux target");

#[cfg(windows)]
mod platform {
    use super::Disk;
    use std::env;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetDiskFreeSpaceExW(
            path: *const u16,
            free: *mut u64,
            total: *mut u64,
            total_free: *mut u64,
        ) -> i32;
    }

    pub fn query() -> Result<Disk, &'static str> {
        let drive = env::var_os("SystemDrive").filter(|drive| !drive.is_empty());
        let drive = drive.as_deref().unwrap_or_else(|| std::ffi::OsStr::new("C:\\"));
        let mut path: Vec<u16> = drive.encode_wide().collect();
        let backslash = u16::from(b'\\');
        if path.last() != Some(&backslash) {
            path.push(backslash);
        }
        path.push(0);

        let mut disk = Disk { free: 0, total: 0 };
        // The path is NUL-terminated and Windows permits a null unused output.
        let success = unsafe {
            GetDiskFreeSpaceExW(path.as_ptr(), &mut disk.free, &mut disk.total, ptr::null_mut())
        };
        if success == 0 {
            return Err("could not query disk space");
        }
        Ok(disk)
    }
}

pub use platform::query;
