//! High-level bindings to the `simavr` AVR simulator.
//!
//! This is a minimal set of high-level bindings that can be used
//! to simulate an AVR MCU from Rust.

extern crate simavr_sys as simavr;
#[macro_use] extern crate bitflags;

pub use self::flashable::{Flashable, Firmware};
pub use simavr_sys as sys;

pub mod uart;
pub mod ioctl;
mod flashable;
#[cfg(test)]
mod test_util;

use std::ffi::{CString, CStr};
use std::mem;
use std::ptr;
use std::os::raw::*;

/// An AVR simulator instance.
pub struct Avr {
    /// The underlying ffi type.
    avr: *mut simavr::avr_t,
    /// The current state.
    current_state: State,
}

/// The status of an AVR mcu.
/// Callbacks will update this structure.
#[derive(Debug, PartialEq, Eq)]
pub struct Status {
    /// Whether we have done the very first initial reset.
    pub powered_on: bool,
    /// The number of times the AVR has reset.
    pub reset_count: u64,
}

/// The state of a simulated AVR.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// Before initialization is finished.
    Limbo = 0,
    /// All is stopped, timers included.
    Stopped,
    /// Running freely.
    Running,
    /// We're sleeping until an interrupt.
    Sleeping,
    /// Run ONE instruction.
    Step,
    /// Tell gdb it's all OK, and give it registers.
    StepDone,
    /// AVR simulation stopped gracefully.
    Done,
    /// AVR simulation crashed (watchdog fired).
    Crashed,
}

bitflags! {
    /// Bitmasks for the flags inside the AVR status register.
    pub struct StatusRegister: u8 {
        /// The interrupt flag (`I`).
        const INTERRUPT_FLAG = 0b10000000;
        /// The transfer bit used by `bld` and `bst` instructions (`T`).
        const TRANSFER_FLAG = 0b01000000;
        /// The half-carry flag (`H`).
        const HALF_CARRY_FLAG = 0b00100000;
        /// The sign flag equal to (`N xor V`) (`S`).
        const SIGN_FLAG = 0b00010000;
        /// The signed overflow flag (`V`)
        const SIGNED_OVERFLOW_FLAG = 0b00001000;
        /// The negative flag (`N`).
        const NEGATIVE_FLAG = 0b00000100;
        /// The zero flag (`Z`).
        const ZERO_FLAG = 0b00000010;
        /// The carry flag (`C`).
        const CARRY_FLAG = 0b00000001;
    }
}

impl Avr {
    /// Contructs an AVR simulator from a raw pointer to a `simavr` simulator.
    pub unsafe fn from_raw(avr: *mut simavr::avr_t) -> Self {
        // Set the simavr global logger hook.
        // We do this upon every new `Avr` object but we could
        // do it once on program initialisation if we wanted.
        simavr::avr_global_logger_set(Some(util::logger));

        let mut avr = Avr {
            avr: avr,
            current_state: State::initial(),
        };

        avr.raw_mut().reset = Some(util::on_reset);

        util::set_mcu_status(Status::default(), avr.avr);
        simavr::avr_init(avr.avr);

        avr.set_frequency(16_000_000);
        avr
    }

    /// Creates a new avr instance.
    pub fn new(mcu_name: &str) -> Result<Self, &'static str> {
        let mcu_name = CString::new(mcu_name).unwrap();
        let avr = unsafe { simavr::avr_make_mcu_by_name(mcu_name.as_ptr()) };

        if avr == ptr::null_mut() {
            return Err("could not create avr sim");
        }

        Ok(unsafe { Avr::from_raw(avr) })
    }

    /// Resets the mcu.
    pub fn reset(&mut self) {
        unsafe {
            simavr::avr_reset(self.avr);
        }
    }

    /// Terminates the mcu.
    pub fn terminate(&mut self) {
        unsafe {
            simavr::avr_terminate(self.avr)
        }
    }

    /// Flashes something to the microcontroller.
    pub fn flash<F>(&mut self, flashable: &F)
        where F: Flashable + ?Sized {
        flashable.flash(self)
    }

    /// Runs a single cycle.
    pub fn run_cycle(&mut self) -> State {
        self.current_state = unsafe {
            simavr::avr_run(self.avr)
        }.into();

        self.current_state
    }

    /// Gets the status of the microcontroller.
    pub fn status(&self) -> &Status {
        unsafe { util::get_mcu_status(self.avr) }
    }

    /// Gets the status register value.
    pub fn status_register(&self) -> StatusRegister {
        println!("sreg: {:?}", self.raw().sreg);
        unimplemented!();
        // StatusRegister::from_bits(0).unwrap()
        // StatusRegister::from_bits(self.raw().sreg).unwrap()
    }

    /// Gets the state of the microcontroller.
    pub fn state(&self) -> &State {
        &self.current_state
    }

    pub fn indefinitely_halted(&self) -> bool {
        unimplemented!();
    }

    /// Gets the name of the mcu.
    pub fn name(&self) -> &str {
        let name = unsafe { CStr::from_ptr(self.raw().mmcu) };
        name.to_str().expect("mcu name is not valid utf-8")
    }

    /// Gets the frequency of the mcu.
    pub fn frequency(&self) -> u32 {
        self.raw().frequency
    }

    /// Sets the frequency of the mcu.
    pub fn set_frequency(&mut self, freq: u32) {
        self.raw_mut().frequency = freq;
    }

    pub unsafe fn underlying(&self) -> *mut simavr::avr_t {
        self.avr
    }

    /// Gets a reference to the underlying `avr_t` structure.
    pub fn raw(&self) -> &simavr::avr_t { unsafe { &*self.avr } }
    /// Gets a mutable reference to the underlying `avr_t` structure.
    pub fn raw_mut(&mut self) -> &mut simavr::avr_t { unsafe { &mut *self.avr } }
}

impl Status {
    /// Whether the microcontroller has been reset *after* it was first started.
    ///
    /// This will ignore the initial reset signal on startup, only considering
    /// resets after startup.
    pub fn has_reset(&self) -> bool { self.reset_count > 0 }
}

impl State {
    /// The very first initial state.
    pub fn initial() -> Self { State::Limbo }

    /// Checks if the state represents a running simulation, regardless
    /// of success of failure.
    pub fn is_running(&self) -> bool {
        match *self {
            State::Limbo => true,
            State::Stopped => false,
            State::Running => true,
            State::Sleeping => true,
            State::Step => true,
            State::StepDone => true,
            State::Done => false,
            State::Crashed => false,
        }
    }
}

impl Drop for Avr {
    fn drop(&mut self) {
        let status: Box<Status> = unsafe {
            Box::from_raw(self.raw().data as *mut Status)
        };

        drop(status)
    }
}

impl Default for Status {
    fn default() -> Status {
        Status {
            powered_on: true,
            reset_count: 0,
        }
    }
}


impl From<c_int> for State {
    fn from(v: c_int) -> Self {
        match v {
            0 => State::Limbo,
            1 => State::Stopped,
            2 => State::Running,
            3 => State::Sleeping,
            4 => State::Step,
            5 => State::StepDone,
            6 => State::Done,
            7 => State::Crashed,
            _ => panic!("unknown state discriminator: {}", v),
        }
    }
}

mod util {
    use super::*;
    use std::io::prelude::*;
    use std::io::stderr;
    use std::process;

    use vsprintf::vsprintf;

    /// Hook that runs when the mcu receives a reset signal.
    pub unsafe extern fn on_reset(avr: *mut simavr::avr_t) {
        let mcu_status = self::get_mcu_status(avr);

        // Check if this is the very first initial reset signal on startup.
        if !mcu_status.powered_on {
            mcu_status.powered_on = true;
        } else {
            // A standard reset.
            mcu_status.reset_count += 1;
        }
    }

    /// The global logger hook.
    /// Gets called on every error/debug/trace/output message.
    pub unsafe extern fn logger(_avr: *mut simavr::avr_t,
                                level: c_int,
                                fmt: *const c_char,
                                args: *mut simavr::__va_list_tag) {
        let message = vsprintf(fmt, args as *mut _).unwrap();
        // Cast the level enum to its Rust bindgen enum.
        let level_kind = mem::transmute(level);

        match level_kind {
            simavr::LOG_NONE => (),
            simavr::LOG_OUTPUT => (),
            simavr::LOG_ERROR => {
                writeln!(stderr(), "error: {}", message).ok();
                process::exit(1);
            },
            simavr::LOG_WARNING => {
                writeln!(stderr(), "warning: {}", message).ok();
            },
            simavr::LOG_TRACE => (),
            simavr::LOG_DEBUG => (),
            _ => (),
        }
    }

    /// Associates a status with a microcontroller.
    pub unsafe fn set_mcu_status(status: Status, avr: *mut simavr::avr_t) {
        let mcu_status = Box::new(status);
        (*avr).data = Box::into_raw(mcu_status) as *mut u8;
    }

    /// Gets the `Status` that is stored inside the custom data field on an `avr_t`.
    pub unsafe fn get_mcu_status<'a>(avr: *mut simavr::avr_t) -> &'a mut Status {
        let ptr = (*avr).data;
        let mut boxed: Box<Status> = Box::from_raw(ptr as *mut Status);

        let status: &'a mut Status = mem::transmute(boxed.as_mut());

        // Forget the box without running the destructors.
        // We only needed to build the box in order to get the underlying
        // reference. The box itself will be freed upon destruction of
        // the `Avr` object.
        mem::forget(boxed);
        status
    }
}

