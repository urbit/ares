use super::Snapshot;
use crate::mem::NockStack;
use crate::mug::mug_u32;
use crate::noun::{Noun, D};
use libc::{c_char, c_int, c_void, size_t};
use std::ffi::CString;
use std::path::{Path, PathBuf};

crate::gdb!();

mod raw {
    use super::*;

    #[repr(C)]
    pub struct RootState {
        pub epoch: u64,
        pub event: u64,
        pub root: u64,
    }

    #[link(name = "pma_malloc", kind = "static")]
    extern "C" {
        pub(super) fn pma_init(path: *const c_char) -> c_int;
        pub(super) fn pma_load(path: *const c_char) -> RootState;
        pub(super) fn pma_close(epoch: u64, event: u64, root: u64) -> c_int;
        pub(super) fn pma_malloc(size: size_t) -> *mut c_void;
        pub(super) fn pma_free(ptr: *mut c_void) -> c_int;
        pub(super) fn pma_sync(epoch: u64, event: u64, root: u64) -> c_int;
        pub(super) fn pma_in_arena(ptr: *const c_void) -> bool;
    }

    #[link(name = "test_pma_malloc_unit", kind = "static")]
    extern "C" {
        pub(super) fn test_pma(path: *const c_char) -> c_void;
    }
}

unsafe fn pma_init<P: AsRef<Path>>(path: P) -> i32 {
    let path = CString::new(path.as_ref().to_str().unwrap()).unwrap();
    raw::pma_init(path.as_ptr())
}

unsafe fn pma_load<P: AsRef<Path>>(path: P) -> (u64, u64, Noun) {
    let path = CString::new(path.as_ref().to_str().unwrap()).unwrap();
    let rs = raw::pma_load(path.as_ptr());
    (rs.epoch, rs.event, Noun::from_raw(rs.root))
}

#[allow(dead_code)]
unsafe fn pma_close(epoch: u64, event: u64, root: Noun) -> i32 {
    raw::pma_close(epoch, event, root.as_raw())
}

pub fn pma_malloc<T>(size: usize) -> *mut T {
    unsafe { raw::pma_malloc(size as size_t) as *mut T }
}

/** Allocate a block of memory from the persistent memory arena.
 *
 * Size is in *words*, unlike the underlying pma_malloc.
 */
pub fn pma_malloc_w<T>(size: usize) -> *mut T {
    unsafe { raw::pma_malloc(size << 3 as size_t) as *mut T }
}

#[allow(dead_code)]
unsafe fn pma_free<T>(ptr: *mut T) -> i32 {
    raw::pma_free(ptr as *mut c_void)
}

unsafe fn pma_sync(epoch: u64, event: u64, root: Noun) -> i32 {
    raw::pma_sync(epoch, event, root.as_raw())
}

pub fn pma_in_arena<T>(ptr: *const T) -> bool {
    unsafe { raw::pma_in_arena(ptr as *const c_void) }
}

#[allow(dead_code)]
unsafe fn test_pma<P: AsRef<Path>>(path: P) {
    let path = CString::new(path.as_ref().to_str().unwrap()).unwrap();
    raw::test_pma(path.as_ptr());
}

pub struct Pma {
    path: PathBuf,
    noun: Noun,
}

impl Pma {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        Self { path, noun: D(0) }
    }
}

impl Snapshot for Pma {
    fn save(&mut self, stack: &mut NockStack, noun: &mut Noun) {
        // Required so everything in the PMA has a cached mug, otherwise we would try to write
        let _mug = mug_u32(stack, *noun);

        unsafe { stack.copy_pma(noun) };
        self.noun = *noun;
    }

    fn sync(&mut self, _stack: &mut NockStack, epoch: u64, event: u64) {
        unsafe {
            pma_sync(epoch, event, self.noun);
        }
    }

    fn load(&mut self, _stack: &mut NockStack) -> std::io::Result<(u64, u64, Noun)> {
        let path = self.path.join(".bin/page.bin");
        if path.is_file() {
            eprintln!("\rload: found snapshot at {:?}", path);
            unsafe { Ok(pma_load(&self.path)) }
        } else {
            eprintln!("\rload: creating snapshot at {:?}", path);
            unsafe { pma_init(&self.path) };
            Ok((0, 0, D(0)))
        }
    }
}