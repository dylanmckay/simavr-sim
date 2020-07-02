//! Testing utilities.

/// Firmwares that can be loaded.
pub mod firmwares {
    pub mod atmega328 {
        use crate::Firmware;

        const FACTORIAL_ELF: &'static [u8] = include_bytes!("atmega328-factorial.elf");

        pub fn factorial() -> Firmware {
            Firmware::read_elf(FACTORIAL_ELF).expect("could not parse factorial elf")
        }
    }
}

