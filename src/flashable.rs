use simavr;

use super::Avr;

use std::io;
use std::io::prelude::*;
use std::ffi::CString;
use std::mem;
use std::path::Path;
use tempfile::NamedTempFile;

/// Something which can be flashes to program memory.
pub trait Flashable {
    fn flash(&self, avr: &mut Avr);
}

/// A handle to a specific AVR firmware image.
pub struct Firmware {
    /// The underlying firmware representation.
    firmware: simavr::elf_firmware_t,
}

impl Firmware {
    /// Create a firmware from its underlying representation.
    pub fn from_raw(firmware: simavr::elf_firmware_t) -> Self {
        Firmware { firmware: firmware }
    }

    /// Reads firmware from an ELF file on disk.
    pub fn read_elf_via_disk<P>(path: P) -> Result<Self, ()>
        where P: AsRef<Path> {
        let path = CString::new(path.as_ref().to_str().unwrap()).unwrap();

        let firmware = unsafe {
            let mut firmware = mem::zeroed();

            let result = simavr::elf_read_firmware(path.as_ptr(), &mut firmware);
            assert_eq!(result, 0, "could not read firmware");
            firmware
        };

        Ok(Firmware::from_raw(firmware))
    }

    /// Reads firmware from an ELF file in memory.
    pub fn read_elf<T>(bytes: T) -> Result<Self, io::Error>
        where T: AsRef<[u8]> {
        let mut tempfile = NamedTempFile::new()?;
        tempfile.write(bytes.as_ref())?;

        Ok(Firmware::read_elf_via_disk(tempfile.path()).unwrap())
    }

    /// Gets the underlying value of the firmware.
    pub fn raw(&self) -> &simavr::elf_firmware_t { &self.firmware }
    /// Gets the underlying value of the firmware.
    pub fn raw_mut(&mut self) -> &mut simavr::elf_firmware_t { &mut self.firmware }
}

impl Flashable for Firmware {
    fn flash(&self, avr: &mut Avr) {
        unsafe {
            simavr::avr_load_firmware(avr.underlying(),
                // This parameter is probably missing a 'const' qualifier
                self.raw() as *const _ as *mut _);
        }
    }
}

impl<T> Flashable for T
    where T: AsRef<[u8]> {

    fn flash(&self, avr: &mut Avr) {
        unsafe {
            let slice = self.as_ref();

            // loadcode array is not const qualified
            let ptr = slice.as_ptr() as *mut u8;
            simavr::avr_loadcode(avr.underlying(), ptr, slice.len() as u32, 0x00);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test_util::firmwares;
    use super::super::State;
    use super::*;

    fn atmega328() -> Avr {
        Avr::with_name("atmega328").unwrap()
    }

    #[test]
    fn can_flash_raw_machine_code() {
        let opcode = [0b1001_0101, 0b1000_1000];
        let mut avr = atmega328();
        avr.flash(&opcode);
        assert_eq!(avr.run_cycle(), State::Running);
    }

    #[test]
    fn can_flash_elf_file() {
        let firmware = firmwares::atmega328::factorial();
        let mut avr = atmega328();
        avr.flash(&firmware);
    }
}

