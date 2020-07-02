// TODO: We could move this into the 'simavr-sys' crate, because
// the macro versions aren't in the generated bindings.

/// Combines bytes together to form an ioctl flag.
pub fn ioctl(a: u8, b: u8, c: u8, d: u8) -> u32 {
    ((a as u32) << 24) |
    ((b as u32) << 16) |
    ((c as u32) << 8) |
    ((d as u32))
}

/// Gets the ioctl flag for a UART with a name character.
pub fn uart(name: char) -> u32 {
    ioctl('u' as u8, 'a' as u8, 'r' as u8, name as u8)
}

pub fn uart_get_flags(name: char) -> u32 {
    ioctl('u' as u8, 'a' as u8, 'g' as u8, name as u8)
}

pub fn uart_set_flags(name: char) -> u32 {
    ioctl('u' as u8, 'a' as u8, 's' as u8, name as u8)
}

