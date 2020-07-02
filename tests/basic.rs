use simavr_sim::*;

fn atmega328() -> Avr {
    Avr::new("atmega328").unwrap()
}

#[test]
fn can_create_mcu() {
    let avr = atmega328();
    assert_eq!(avr.name(), "atmega328");
}

#[test]
fn new_mcu_status_is_default() {
    let avr = atmega328();
    assert_eq!(avr.status(), &Status::default());
}

#[test]
fn first_initialise_increments_reset_count() {
    let mut avr = atmega328();

    assert_eq!(avr.status().reset_count, 0);
    avr.run_cycle();
    assert_eq!(avr.status().reset_count, 0);
}

#[test]
fn explicit_resets_after_first_increment_reset_count() {
    let mut avr = atmega328();

    assert_eq!(avr.status().reset_count, 0);

    // Run a few cycles to for good measure.
    for _ in 0..4 {
        avr.run_cycle();
        assert_eq!(avr.status().reset_count, 0);
    }

    avr.reset();
    assert_eq!(avr.status().reset_count, 1);
    avr.reset();
    assert_eq!(avr.status().reset_count, 2);
}

#[test]
fn has_reset_makes_sense() {
    let mut avr = atmega328();
    assert_eq!(avr.status().has_reset(), false);

    // Run a few cycles for good measure.
    for _ in 0..4 {
        avr.run_cycle();
        assert_eq!(avr.status().has_reset(), false);
    }

    avr.reset();
    assert_eq!(avr.status().has_reset(), true);
}

#[test]
fn initial_state_matches_expected() {
    let avr = atmega328();
    assert_eq!(avr.state(), &State::initial());
}

#[test]
fn state_is_running_after_first_cycle() {
    let mut avr = atmega328();
    avr.run_cycle();
    assert_eq!(avr.state(), &State::Running);
}
