//! Label — disk label (PT) specific data and functions

use crate::context::Context;
use crate::errors::*;
use fdisk_sys;
use std::ffi::{CStr, CString};

/// Container for fdisk partitions
pub struct Label {
    pub(crate) ptr: *mut fdisk_sys::fdisk_label,
}
#[repr(u32)]
pub enum DiskLabel {
    Dos = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_DOS,
    Sun = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_SUN,
    Sgi = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_SGI,
    Bsd = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_BSD,
    Gpt = fdisk_sys::fdisk_labeltype_FDISK_DISKLABEL_GPT,
}

impl AsRef<str> for DiskLabel {
    fn as_ref(&self) -> &str {
        match self {
            DiskLabel::Dos => "dos",
            DiskLabel::Sun => "sun",
            DiskLabel::Sgi => "sgi",
            DiskLabel::Gpt => "gpt",
            DiskLabel::Bsd => "bsd",
        }
    }
}

impl ToString for DiskLabel {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

impl Label {
    pub fn get_name(&self) -> Result<String> {
        unsafe {
            let src = fdisk_sys::fdisk_label_get_name(self.ptr);
            if src.is_null() {
                return Err("no valid name".into());
            }
            match CStr::from_ptr(src).to_str() {
                Ok(v) => Ok(v.to_string()),
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Returns `true` if label driver disabled
    pub fn is_disabled(&self) -> bool {
        unsafe { fdisk_sys::fdisk_label_is_disabled(self.ptr) == 1 }
    }

    /// Returns `true` if in-memory data has been changed
    pub fn is_changed(&self) -> bool {
        unsafe { fdisk_sys::fdisk_label_is_changed(self.ptr) == 1 }
    }
}

impl Context {
    /// Return 'true' if list-only mode enabled
    /// # Arguments
    /// * `id`- FDISK_DISKLABEL_*
    pub fn is_labeltype(&self, id: DiskLabel) -> bool {
        match unsafe { fdisk_sys::fdisk_is_labeltype(self.ptr, id as u32) } {
            1 => true,
            _ => false,
        }
    }

    /// Creates a new disk label of type name .
    /// If name is NULL, then it will create a default system label type, either SUN or DOS.
    pub fn create_disklabel<L: AsRef<str>>(&self, name: L) -> Result<()> {
        let name = CString::new(name.as_ref().as_bytes())?;
        match unsafe { fdisk_sys::fdisk_create_disklabel(self.ptr, name.as_ptr()) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// Write in-memory changes to disk.
    pub fn write_disklabel(&self) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_write_disklabel(self.ptr) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// Verifies the partition table.
    pub fn verify_disklabel(&self) -> Result<()> {
        match unsafe { fdisk_sys::fdisk_verify_disklabel(self.ptr) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// If no name specified then returns the current context label.
    pub fn get_label<L: AsRef<str>>(&self, name: L) -> Result<Label> {
        let name = match name.as_ref().is_empty() {
            false => CString::new(name.as_ref().as_bytes())?.as_ptr(),
            true => std::ptr::null(),
        };
        unsafe {
            let ptr = fdisk_sys::fdisk_get_label(self.ptr, name);
            if ptr.is_null() {
                return Err("no valid label".into());
            }
            Ok(Label { ptr })
        }
    }

    /// Return 'true' if there is label on the device.
    pub fn has_label(&self) -> bool {
        match unsafe { fdisk_sys::fdisk_has_label(self.ptr) } {
            1 => true,
            _ => false,
        }
    }
}
