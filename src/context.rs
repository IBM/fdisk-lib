//! Stores info about device, labels etc.

use crate::errors::*;
use crate::table::Table;
use fdisk_sys;
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[repr(u32)]
pub enum DiskLabel {
    Dos = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_DOS,
    Sun = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_SUN,
    Sgi = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_SGI,
    Bsd = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_BSD,
    Gpt = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_GPT,
}

#[repr(u32)]
pub enum DiskUnit {
    Human = fdisk_sys::FDISK_SIZEUNIT_HUMAN,
    Bytes = fdisk_sys::FDISK_SIZEUNIT_BYTES,
}

/// Stores info about device
pub struct Context {
    pub(crate) ptr: *mut fdisk_sys::fdisk_context,
}

impl Context {
    /// Returns a new context for libfdisk
    pub fn new() -> Context {
        Context {
            ptr: unsafe { fdisk_sys::fdisk_new_context() },
        }
    }

    /// Create a new nested fdisk context for nested disk labels (e.g. BSD or PMBR).
    /// The function also probes for the nested label on the device if
    /// device is already assigned to parent. The new context is initialized according
    /// to parent and both context shares some settings and file descriptor to the device.
    /// The child propagate some changes (like fdisk_assign_device()) to parent,
    /// but it does not work vice-versa.
    /// # Arguments
    /// * `name` - optional label name (e.g. "bsd")
    pub fn new_nested(&self, name: &str) -> Result<Context> {
        let name = CString::new(name.as_bytes())?;
        let ptr = unsafe { fdisk_sys::fdisk_new_nested_context(self.ptr, name.as_ptr()) };
        if ptr.is_null() {
            return Err(nix::Error::last().into());
        }
        Ok(Context { ptr })
    }

    /// Increments reference counter.
    pub fn ref_context(&self) {
        unsafe { fdisk_sys::fdisk_ref_context(self.ptr) }
    }

    /// Open the device, discovery topology, geometry, detect disklabel and switch
    /// the current label driver to reflect the probing result.
    /// # Arguments
    /// * `name` - path to the device to be handled
    /// * `readonly` - how to open the device
    pub fn assign_device<P: AsRef<Path>>(&self, name: P, readonly: bool) -> Result<()> {
        let device = CString::new(name.as_ref().as_os_str().as_bytes())
            .chain_err(|| format!("converting to CString {}", name.as_ref().display()))?;
        match unsafe { fdisk_sys::fdisk_assign_device(self.ptr, device.as_ptr(), readonly as i32) }
        {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// Close device  and call fsync(). If the cxt is nested context
    /// than the request is redirected to the parent.
    /// # Arguments
    /// * `nosync` - disable fsync()
    pub fn deassign_device(&self, nosync: bool) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_deassign_device(self.ptr, nosync as i32) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// The library zeroizes all the first sector when create a new disk label by default.
    /// This function allows to control this behavior. For now it's supported for MBR and GPT.
    /// # Arguments
    //  * `enable` - true or false
    pub fn enable_bootbits_protection(&self, enable: bool) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_enable_bootbits_protection(self.ptr, enable as i32) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// Enables or disables "details" display mode.
    /// This function has effect to partition_to_string() function.
    /// # Arguments
    //  * `enable` - true or false
    pub fn enable_details(&self, enable: bool) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_enable_details(self.ptr, enable as i32) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// Just list partition only, don't care about another details, mistakes, ...
    /// # Arguments
    //  * `enable` - true or false
    pub fn enable_listonly(&self, enable: bool) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_enable_listonly(self.ptr, enable as i32) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// The alignment offset is offset between logical and physical sectors.
    /// For backward compatibility the first logical sector on 4K disks does
    /// no have to start on the same place like physical sectors.
    pub fn alignment_offset(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_alignment_offset(self.ptr) }
    }

    /// Return device file descriptor.
    pub fn fd(&self) -> i32 {
        unsafe { fdisk_sys::fdisk_get_devfd(self.ptr) }
    }

    /// Return device name.
    pub fn name(&self) -> Result<String> {
        unsafe {
            let src = fdisk_sys::fdisk_get_devname(self.ptr);
            if src.is_null() {
                return Err("no valid name".into());
            }
            match CStr::from_ptr(src).to_str() {
                Ok(v) => Ok(v.to_string()),
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Return first possible LBA on disk for data partitions.
    pub fn first_lba(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_first_lba(self.ptr) }
    }

    /// Return number of geometry cylinders.
    pub fn cylinders(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_geom_cylinders(self.ptr) }
    }

    /// Return number of geometry heads.
    pub fn heads(&self) -> u32 {
        unsafe { fdisk_sys::fdisk_get_geom_heads(self.ptr) }
    }

    /// Return number of geometry sectors.
    pub fn sectors(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_geom_sectors(self.ptr) }
    }

    /// Return grain in bytes used to align partitions (usually 1MiB)
    pub fn grain(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_grain_size(self.ptr) }
    }

    /// Return flast possible LBA on device.
    pub fn last_lba(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_last_lba(self.ptr) }
    }

    /// Return minimal I/O size in bytes.
    pub fn minimal_io_size(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_minimal_iosize(self.ptr) }
    }

    /// Return size of the device in logical sectors.
    pub fn logical_sectors(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_nsectors(self.ptr) }
    }

    /// Return The optimal I/O is optional and does not have to be provided by device,
    /// anyway libfdisk never returns zero. If the optimal I/O size is not provided
    /// then libfdisk returns minimal I/O size or sector size.
    pub fn optimal_io_size(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_optimal_iosize(self.ptr) }
    }

    /// Return parental context
    pub fn parent(&self) -> Option<Context> {
        unsafe {
            let ptr = fdisk_sys::fdisk_get_parent(self.ptr);
            if ptr.is_null() {
                return None;
            }
            Some(Context { ptr })
        }
    }

    /// Return physical sector size in bytes
    pub fn phy_sector_size(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_physector_size(self.ptr) }
    }

    /// Return logical sector size in bytes
    pub fn sector_size(&self) -> u64 {
        unsafe { fdisk_sys::fdisk_get_sector_size(self.ptr) }
    }

    /// Add partitions from disklabel to table
    pub fn get_partitions(&self) -> Result<Table> {
        let mut table = Table::new();
        match unsafe { fdisk_sys::fdisk_get_partitions(self.ptr, &mut table.ptr) } {
            0 => Ok(table),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// Return unit for SIZE output field
    pub fn unit_size(&self) -> i32 {
        unsafe { fdisk_sys::fdisk_get_size_unit(self.ptr) }
    }

    /// Return unit name.
    /// # Arguments
    /// * `singular` - false (FDISK_PLURAL) or true (FDISK_SINGULAR)
    pub fn unit(&self, singular: bool) -> Result<String> {
        let n = if singular {
            fdisk_sys::FDISK_SINGULAR
        } else {
            fdisk_sys::FDISK_PLURAL
        };
        unsafe {
            let src = fdisk_sys::fdisk_get_unit(self.ptr, n as i32);
            if src.is_null() {
                return Err("no valid name".into());
            }
            match CStr::from_ptr(src).to_str() {
                Ok(v) => Ok(v.to_string()),
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Return number of "units" per sector, default is 1 if display unit is sector.
    /// This is necessary only for brain dead situations when we use "cylinders"
    pub fn units_per_sector(&self) -> u32 {
        unsafe { fdisk_sys::fdisk_get_units_per_sector(self.ptr) }
    }

    /// Return 'true' if there is label on the device.
    pub fn has_label(&self) -> bool {
        match unsafe { fdisk_sys::fdisk_has_label(self.ptr) } {
            1 => true,
            _ => false,
        }
    }

    /// Return 'true' if boot bits protection enabled.
    pub fn has_protected_bootbits(&self) -> bool {
        match unsafe { fdisk_sys::fdisk_has_protected_bootbits(self.ptr) } {
            1 => true,
            _ => false,
        }
    }

    /// Return 'true' if details are enabled
    pub fn is_details(&self) -> bool {
        match unsafe { fdisk_sys::fdisk_is_details(self.ptr) } {
            1 => true,
            _ => false,
        }
    }

    /// Return 'true' if list-only mode enabled
    /// # Arguments
    /// * `id`- FDISK_DISKLABEL_*
    pub fn is_labeltype(&self, id: DiskLabel) -> bool {
        match unsafe { fdisk_sys::fdisk_is_labeltype(self.ptr, id as u32) } {
            1 => true,
            _ => false,
        }
    }

    /// Return 'true' if list-only mode enabled
    pub fn is_listonly(&self) -> bool {
        match unsafe { fdisk_sys::fdisk_is_listonly(self.ptr) } {
            1 => true,
            _ => false,
        }
    }

    /// Return 'true' if device open readonly
    pub fn is_readonly(&self) -> bool {
        match unsafe { fdisk_sys::fdisk_is_readonly(self.ptr) } {
            1 => true,
            _ => false,
        }
    }

    /// It's strongly recommended to use the default library setting.
    /// The first LBA is always reseted by assign_device(),
    /// override_geometry() and reset_alignment().
    /// This is very low level function and library does not check if your setting makes any sense.
    /// This function is necessary only when you want to work with very unusual partition tables
    /// like GPT protective MBR or hybrid partition tables on bootable media where
    /// the first partition may start on very crazy offsets.
    ///
    /// # Arguments
    /// * `lba` - first possible logical sector for data
    pub fn set_first_lba(&self, lba: u64) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_set_first_lba(self.ptr, lba) } {
            0 => Ok(()),
            v => Err(format!("fdisk_set_first_lba failed with code {}", v).into()),
        }
    }

    /// It's strongly recommended to use the default library setting.
    /// The last LBA is always reseted by assign_device(), override_geometry()
    /// and reset_alignment(). The default is number of sectors on the device,
    /// but maybe modified by the current disklabel driver (for example GPT uses the
    /// end of disk for backup header, so last_lba is smaller than total number of sectors).
    ///
    /// # Arguments
    /// * `lba` - last possible logical sector for data
    pub fn set_last_lba(&self, lba: u64) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_set_last_lba(self.ptr, lba) } {
            0 => Ok(()),
            v => Err(format!("fdisk_set_last_lba failed with code {}", v).into()),
        }
    }

    /// Sets unit for SIZE output field (see fdisk_partition_to_string()).
    /// # Arguments
    /// * `unit` - DiskUnit
    pub fn set_size_unit(&self, unit: DiskUnit) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_set_size_unit(self.ptr, unit as i32) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// For example Sun addresses begin of the partition by cylinders...
    /// # Arguments
    /// * `cylinders` - true(display in cylinders) or false (display in sectors)
    pub fn set_unit(&self, cylinders: bool) -> Result<()> {
        let s = if cylinders {
            CString::new("cylinder")?
        } else {
            CString::new("sector")?
        };
        match unsafe { fdisk_sys::fdisk_set_unit(self.ptr, s.as_ptr()) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// Return 1 if user wants to display in cylinders.
    pub fn use_cylinders(&self) -> i32 {
        unsafe { fdisk_sys::fdisk_use_cylinders(self.ptr) }
    }

    /// Save user defined sector sizes to use it for partitioning
    ///
    /// # Arguments
    /// * `phy` - physical sector size
    /// * `log` - logical sector size
    pub fn save_user_sector_size(&self, phy: u32, log: u32) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_save_user_sector_size(self.ptr, phy, log) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { fdisk_sys::fdisk_unref_context(self.ptr) }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
