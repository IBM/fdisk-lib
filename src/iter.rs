//! Unified iterator.
//! The iterator keeps the direction and the last position for access
//! to the internal library tables/lists.
//!
use crate::partition::Partition;
use crate::table::Table;
use fdisk_sys;

/// Unified iterator
pub struct Iter<'a> {
    tbl: &'a mut Table,
    ptr: *mut fdisk_sys::fdisk_iter,
}

impl<'a> Iter<'a> {
    pub fn new(tbl: &mut Table) -> Iter {
        Iter {
            tbl,
            ptr: unsafe { fdisk_sys::fdisk_new_iter(fdisk_sys::FDISK_ITER_FORWARD as i32) },
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Partition;

    fn next(&mut self) -> Option<Self::Item> {
        let mut ptr: *mut fdisk_sys::fdisk_partition = std::ptr::null_mut();
        match unsafe { fdisk_sys::fdisk_table_next_partition(self.tbl.ptr, self.ptr, &mut ptr) } {
            0 => Some(Partition { ptr }),
            1 => None,
            _ => panic!("bad value"),
        }
    }
}

impl<'a> Drop for Iter<'a> {
    fn drop(&mut self) {
        unsafe { fdisk_sys::fdisk_free_iter(self.ptr) }
    }
}
