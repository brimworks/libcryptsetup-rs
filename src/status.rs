use std::{convert::TryFrom, os::raw::c_int, path::Path, str::FromStr};

use crate::{
    device::CryptDevice,
    err::LibcryptErr,
    format::{CryptParamsIntegrity, CryptParamsVerity},
};

use libcryptsetup_rs_sys as cryptsetup_sys;

use uuid::Uuid;

pub enum CryptStatusInfo {
    Invalid = cryptsetup_sys::crypt_status_info_CRYPT_INVALID as isize,
    Inactive = cryptsetup_sys::crypt_status_info_CRYPT_INACTIVE as isize,
    Active = cryptsetup_sys::crypt_status_info_CRYPT_ACTIVE as isize,
    Busy = cryptsetup_sys::crypt_status_info_CRYPT_BUSY as isize,
}

impl TryFrom<u32> for CryptStatusInfo {
    type Error = LibcryptErr;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        Ok(match v {
            i if i == CryptStatusInfo::Invalid as u32 => CryptStatusInfo::Invalid,
            i if i == CryptStatusInfo::Inactive as u32 => CryptStatusInfo::Inactive,
            i if i == CryptStatusInfo::Active as u32 => CryptStatusInfo::Active,
            i if i == CryptStatusInfo::Busy as u32 => CryptStatusInfo::Busy,
            _ => return Err(LibcryptErr::InvalidConversion),
        })
    }
}

/// Handle for crypt device status operations
pub struct CryptDeviceStatus<'a> {
    reference: &'a mut CryptDevice,
}

impl<'a> CryptDeviceStatus<'a> {
    pub(crate) fn new(reference: &'a mut CryptDevice) -> Self {
        CryptDeviceStatus { reference }
    }

    /// Get status info from device name
    pub fn status(&mut self, name: &str) -> Result<CryptStatusInfo, LibcryptErr> {
        let name_cstring = to_cstring!(name)?;
        try_int_to_return!(
            unsafe { cryptsetup_sys::crypt_status(self.reference.as_ptr(), name_cstring.as_ptr()) },
            CryptStatusInfo
        )
    }

    /// Dump text info about device to log output
    pub fn dump(&mut self) -> Result<(), LibcryptErr> {
        errno!(unsafe { cryptsetup_sys::crypt_dump(self.reference.as_ptr()) })
    }

    /// Get cipher used by device
    pub fn get_cipher(&mut self) -> Result<String, LibcryptErr> {
        from_str_ptr_to_owned!(cryptsetup_sys::crypt_get_cipher(self.reference.as_ptr()))
    }

    /// Get cipher mode used by device
    pub fn get_cipher_mode(&mut self) -> Result<String, LibcryptErr> {
        from_str_ptr_to_owned!(cryptsetup_sys::crypt_get_cipher_mode(
            self.reference.as_ptr()
        ))
    }

    /// Get device UUID
    pub fn get_uuid(&mut self) -> Result<Uuid, LibcryptErr> {
        from_str_ptr!(cryptsetup_sys::crypt_get_uuid(self.reference.as_ptr()))
            .and_then(|e| Uuid::from_str(e).map_err(LibcryptErr::UuidError))
    }

    /// Get path to underlying device
    pub fn get_device_path(&mut self) -> Result<&Path, LibcryptErr> {
        from_str_ptr!(cryptsetup_sys::crypt_get_device_name(
            self.reference.as_ptr()
        ))
        .map(Path::new)
    }

    /// Get path to detached metadata device or `None` if it is attached
    pub fn get_metadata_device_path(&mut self) -> Result<Option<&Path>, LibcryptErr> {
        let ptr =
            unsafe { cryptsetup_sys::crypt_get_metadata_device_name(self.reference.as_ptr()) };
        if ptr.is_null() {
            return Ok(None);
        }
        from_str_ptr!(ptr).map(|s| Some(Path::new(s)))
    }

    /// Get offset in 512-byte sectors where real data starts
    pub fn get_data_offset(&mut self) -> u64 {
        unsafe { cryptsetup_sys::crypt_get_data_offset(self.reference.as_ptr()) }
    }

    /// Get IV location offset in 512-byte sectors
    pub fn get_iv_offset(&mut self) -> u64 {
        unsafe { cryptsetup_sys::crypt_get_iv_offset(self.reference.as_ptr()) }
    }

    /// Get size in bytes of volume key
    pub fn get_volume_key_size(&mut self) -> c_int {
        unsafe { cryptsetup_sys::crypt_get_volume_key_size(self.reference.as_ptr()) }
    }

    /// Get size of encryption sectors in bytes
    pub fn get_sector_size(&mut self) -> c_int {
        unsafe { cryptsetup_sys::crypt_get_sector_size(self.reference.as_ptr()) }
    }

    /// Get Verity device parameters
    pub fn get_verity_info(&mut self) -> Result<CryptParamsVerity, LibcryptErr> {
        let mut verity = cryptsetup_sys::crypt_params_verity {
            hash_name: std::ptr::null(),
            data_device: std::ptr::null(),
            hash_device: std::ptr::null(),
            fec_device: std::ptr::null(),
            salt: std::ptr::null(),
            salt_size: 0,
            hash_type: 0,
            data_block_size: 0,
            hash_block_size: 0,
            data_size: 0,
            hash_area_offset: 0,
            fec_area_offset: 0,
            fec_roots: 0,
            flags: 0,
        };
        errno!(unsafe {
            cryptsetup_sys::crypt_get_verity_info(self.reference.as_ptr(), &mut verity as *mut _)
        })
        .and_then(|_| CryptParamsVerity::try_from(&verity))
    }

    /// Get Integrity device parameters
    pub fn get_integrity_info(&mut self) -> Result<CryptParamsIntegrity, LibcryptErr> {
        let mut integrity = cryptsetup_sys::crypt_params_integrity {
            journal_size: 0,
            journal_watermark: 0,
            journal_commit_time: 0,
            interleave_sectors: 0,
            tag_size: 0,
            sector_size: 0,
            buffer_sectors: 0,
            integrity: std::ptr::null(),
            integrity_key_size: 0,
            journal_integrity: std::ptr::null(),
            journal_integrity_key: std::ptr::null(),
            journal_integrity_key_size: 0,
            journal_crypt: std::ptr::null(),
            journal_crypt_key: std::ptr::null(),
            journal_crypt_key_size: 0,
        };
        errno!(unsafe {
            cryptsetup_sys::crypt_get_integrity_info(
                self.reference.as_ptr(),
                &mut integrity as *mut _,
            )
        })
        .and_then(|_| CryptParamsIntegrity::try_from(&integrity))
    }
}
