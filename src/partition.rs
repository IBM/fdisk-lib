//! Generic label independent partition abstraction.
//! Provides label independent abstraction. The partitions are not directly connected with
//! partition table (label) data. Any change to fdisk_partition does not affects in-memory
//! or on-disk label data. The fdisk_partition is possible to use as a
//! template for fdisk_add_partition() or fdisk_set_partition() operations.

use crate::context::Context;
use anyhow::{anyhow, Result};
use fdisk_sys;
use std::ffi::{CStr, CString};

/// Generic label independent partition abstraction
pub struct Partition {
    pub(crate) ptr: *mut fdisk_sys::fdisk_partition,
}

impl Partition {
    /// Return newly allocated Partition
    pub fn new() -> Partition {
        Partition {
            ptr: unsafe { fdisk_sys::fdisk_new_partition() },
        }
    }

    /// Increment reference counter.
    pub fn ref_partition(&self) {
        unsafe { fdisk_sys::fdisk_ref_partition(self.ptr) }
    }

    /// Reset partition content.
    pub fn reset_partition(&self) {
        unsafe { fdisk_sys::fdisk_reset_partition(self.ptr) }
    }

    /// Return partition attributes in string format
    pub fn attrs(&self) -> Option<String> {
        unsafe {
            let ptr = fdisk_sys::fdisk_partition_get_attrs(self.ptr);
            if ptr.is_null() {
                return None;
            }
            Some(CStr::from_ptr(ptr).to_str().unwrap().to_string())
        }
    }

    /// Return last partition sector LBA.
    pub fn end(&self) -> Option<u64> {
        match unsafe { fdisk_sys::fdisk_partition_has_end(self.ptr) } {
            0 => None,
            _ => Some(unsafe { fdisk_sys::fdisk_partition_get_end(self.ptr) }),
        }
    }

    /// Return partition UUID as string
    pub fn name(&self) -> Result<String> {
        unsafe {
            let src = fdisk_sys::fdisk_partition_get_name(self.ptr);
            if src.is_null() {
                return Err(anyhow!("no valid Name"));
            }
            match CStr::from_ptr(src).to_str() {
                Ok(v) => Ok(v.to_string()),
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Return devno of the parent
    pub fn parent(&self) -> Result<usize> {
        let mut p: usize = 0;
        match unsafe { fdisk_sys::fdisk_partition_get_parent(self.ptr, &mut p) } {
            0 => Ok(p),
            v => Err(anyhow!(
                "getting parent, errno: {}",
                nix::errno::from_i32(v)
            )),
        }
    }

    /// Return partition number (0 is the first partition)
    pub fn partno(&self) -> Option<usize> {
        match unsafe { fdisk_sys::fdisk_partition_has_partno(self.ptr) } {
            0 => None,
            _ => Some(unsafe { fdisk_sys::fdisk_partition_get_partno(self.ptr) }),
        }
    }

    /// Return size offset in sectors
    pub fn size(&self) -> Option<u64> {
        match unsafe { fdisk_sys::fdisk_partition_has_size(self.ptr) } {
            0 => None,
            _ => Some(unsafe { fdisk_sys::fdisk_partition_get_size(self.ptr) }),
        }
    }

    /// Return start offset in sectors
    pub fn start(&self) -> Option<u64> {
        match unsafe { fdisk_sys::fdisk_partition_has_start(self.ptr) } {
            0 => None,
            _ => Some(unsafe { fdisk_sys::fdisk_partition_get_start(self.ptr) }),
        }
    }

    /// Return partition UUID as string
    pub fn uuid(&self) -> Result<String> {
        unsafe {
            let src = fdisk_sys::fdisk_partition_get_uuid(self.ptr);
            if src.is_null() {
                return Err(anyhow!("no valid UUID"));
            }
            match CStr::from_ptr(src).to_str() {
                Ok(v) => Ok(v.to_string()),
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Return true if the partition has enabled boot flag
    pub fn is_bootable(&self) -> bool {
        matches!(
            unsafe { fdisk_sys::fdisk_partition_is_bootable(self.ptr) },
            1
        )
    }

    /// Return true if the partition is container (e.g. MBR extended partition)
    pub fn is_container(&self) -> bool {
        matches!(
            unsafe { fdisk_sys::fdisk_partition_is_container(self.ptr) },
            1
        )
    }

    /// Return true if points to freespace
    pub fn is_freespace(&self) -> bool {
        matches!(
            unsafe { fdisk_sys::fdisk_partition_is_freespace(self.ptr) },
            1
        )
    }

    /// Return true if the partition is nested (e.g. MBR logical partition)
    pub fn is_nested(&self) -> bool {
        matches!(unsafe { fdisk_sys::fdisk_partition_is_nested(self.ptr) }, 1)
    }

    /// Return true if the partition points to some area
    pub fn is_used(&self) -> bool {
        matches!(unsafe { fdisk_sys::fdisk_partition_is_used(self.ptr) }, 1)
    }

    /// Return true if the partition is special whole-disk (e.g. SUN) partition
    pub fn is_wholedisk(&self) -> bool {
        matches!(
            unsafe { fdisk_sys::fdisk_partition_is_wholedisk(self.ptr) },
            1
        )
    }

    pub fn set_partno(&self, partno: usize) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_partition_set_partno(self.ptr, partno) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting partno '{}', errno: {}",
                partno,
                nix::errno::from_i32(v)
            )),
        }
    }

    pub fn set_size(&self, size: u64) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_partition_set_size(self.ptr, size) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting size '{}', errno: {}",
                size,
                nix::errno::from_i32(v)
            )),
        }
    }

    pub fn set_start(&self, start: u64) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_partition_set_start(self.ptr, start) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting start '{}', errno: {}",
                start,
                nix::errno::from_i32(v)
            )),
        }
    }

    pub fn set_attrs(&self, attrs: &str) -> Result<()> {
        let value = match CString::new(attrs.as_bytes()) {
            Ok(s) => s,
            _ => return Err(anyhow!("converting {} to CString", attrs)),
        };
        match unsafe { fdisk_sys::fdisk_partition_set_attrs(self.ptr, value.as_ptr()) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting attributes '{}', errno: {}",
                attrs,
                nix::errno::from_i32(v)
            )),
        }
    }

    pub fn set_name(&self, name: &str) -> Result<()> {
        let value = match CString::new(name.as_bytes()) {
            Ok(s) => s,
            _ => return Err(anyhow!("converting {} to CString", name)),
        };
        match unsafe { fdisk_sys::fdisk_partition_set_name(self.ptr, value.as_ptr()) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting name '{}', errno: {}",
                name,
                nix::errno::from_i32(v)
            )),
        }
    }

    pub fn set_uuid(&self, uuid: &str) -> Result<()> {
        let value = match CString::new(uuid.as_bytes()) {
            Ok(s) => s,
            _ => return Err(anyhow!("converting {} to CString", uuid)),
        };
        match unsafe { fdisk_sys::fdisk_partition_set_uuid(self.ptr, value.as_ptr()) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting uuid '{}', errno: {}",
                uuid,
                nix::errno::from_i32(v)
            )),
        }
    }

    /// By default libfdisk aligns the size when add the new partition (by add_partition()).
    /// If you want to disable this functionality use enable = true.
    pub fn size_explicit(&self, enable: bool) -> Result<()> {
        match unsafe {
            fdisk_sys::fdisk_partition_size_explicit(self.ptr, if enable { 1 } else { 0 })
        } {
            0 => Ok(()),
            v => Err(anyhow!(
                "changing size explicit, errno: {}",
                nix::errno::from_i32(v)
            )),
        }
    }

    /// When partition used as a template for add_partition() when force label driver
    pub fn start_follow_default(&self, enable: bool) -> Result<()> {
        match unsafe {
            fdisk_sys::fdisk_partition_start_follow_default(self.ptr, if enable { 1 } else { 0 })
        } {
            0 => Ok(()),
            v => Err(anyhow!(
                "changing follow defaults, errno: {}",
                nix::errno::from_i32(v)
            )),
        }
    }

    /// Return true if the partition follows default
    pub fn start_is_default(&self) -> bool {
        matches!(
            unsafe { fdisk_sys::fdisk_partition_start_is_default(self.ptr) },
            1
        )
    }

    /// Sets the partno as undefined.
    pub fn unset_partno(&self) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_partition_unset_partno(self.ptr) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting partno as undefined, errno: {}",
                nix::errno::from_i32(v)
            )),
        }
    }

    /// Sets the size as undefined
    pub fn unset_size(&self) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_partition_unset_size(self.ptr) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting size as undefined, errno: {}",
                nix::errno::from_i32(v)
            )),
        }
    }

    /// Sets the start as undefined
    pub fn unset_start(&self) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_partition_unset_start(self.ptr) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting start as undefined, errno: {}",
                nix::errno::from_i32(v)
            )),
        }
    }
}

impl Drop for Partition {
    fn drop(&mut self) {
        unsafe { fdisk_sys::fdisk_unref_partition(self.ptr) }
    }
}

impl Default for Partition {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Modifies disklabel according to setting with in pa .
    /// # Arguments
    /// * `partno` - partition number (0 is the first partition)
    /// * `pt` - new partition setting
    pub fn set_partition(&self, no: usize, pt: &Partition) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_set_partition(self.ptr, no, pt.ptr) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "setting partno '{}', errno: {}",
                no,
                nix::errno::from_i32(v)
            )),
        }
    }

    /// Delete all used partitions from disklabel
    pub fn delete_all_partitions(&self) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_delete_all_partitions(self.ptr) } {
            0 => Ok(()),
            v => Err(anyhow!(
                "deleting all partitions, errno: {}",
                nix::errno::from_i32(v)
            )),
        }
    }
}
