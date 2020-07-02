//! Logic relating to UART serial communication.

use crate::Avr;
use super::ioctl;

use simavr;
use std::os::raw::c_void;

use std::ffi::CString;
use std::ptr;

/// The names of the IRQs we want to attach to.
const IRQ_NAMES: &'static [&'static str] = &[
    "8<uart_pty.in", // Must be first
    "8>uart_pty.out",
];

/// Attaches the AVR UART to the current standard output stream.
pub fn attach_to_stdout(avr: &mut Avr) {
    let irq_names: Vec<_> = IRQ_NAMES.iter()
                                      .map(|&irq| CString::new(irq).unwrap())
                                      .collect();

    let mut irq_names: Vec<_> = irq_names.iter().map(|irq| irq.as_ptr()).collect();

    unsafe {
        let irq = simavr::avr_alloc_irq(&mut avr.raw_mut().irq_pool, 0,
            irq_names.len() as u32, irq_names.as_mut_ptr());

        let param = ptr::null_mut();
        simavr::avr_irq_register_notify(irq, Some(self::irq_input_hook), param);

        let uart_name = '0';
        let uart = ioctl::uart(uart_name);

        // Disable dumping of stdout.
        let mut stdio_flag: u32 = 0;
        simavr::avr_ioctl(avr.underlying(), ioctl::uart_get_flags(uart_name), &mut stdio_flag as *mut u32 as *mut _);
        stdio_flag &= !(simavr::AVR_UART_FLAG_STDIO as u32);
        simavr::avr_ioctl(avr.underlying(), ioctl::uart_set_flags(uart_name), &mut stdio_flag as *mut u32 as *mut _);

        let src = simavr::avr_io_getirq(avr.raw_mut() as *mut _, uart, simavr::UART_IRQ_OUTPUT as _);
        let dst = simavr::avr_io_getirq(avr.raw_mut() as *mut _, uart, simavr::UART_IRQ_INPUT as _);

        if src != ptr::null_mut() && dst != ptr::null_mut() {
            simavr::avr_connect_irq(src, irq);
        }
    }
}

unsafe extern "C" fn irq_input_hook(_irq: *mut simavr::avr_irq_t,
                                    value: u32,
                                    _param: *mut c_void) {
    print!("{}", value as u8 as char);
}
