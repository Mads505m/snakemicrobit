use core::cell::RefCell;
use cortex_m::interrupt::{Mutex, free};
use microbit::hal::gpiote::Gpiote;
use microbit::pac::{self, GPIOTE, interrupt};
use microbit::board::Buttons;
use crate::game::Turn;

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static TURN: Mutex<RefCell<Turn>> = Mutex::new(RefCell::new(Turn::None));

/// Initialize buttons A and B with falling-edge interrupts.
pub(crate) fn init_buttons(board_gpiote: GPIOTE, board_buttons: Buttons) {
    let gpiote = Gpiote::new(board_gpiote);

    let channel0 = gpiote.channel0();
    channel0
        .input_pin(&board_buttons.button_a.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel0.reset_events();

    let channel1 = gpiote.channel1();
    channel1
        .input_pin(&board_buttons.button_b.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel1.reset_events();

    free(move |cs| {
        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);
        unsafe { pac::NVIC::unmask(pac::Interrupt::GPIOTE); }
        pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    });
}

/// Interrupt handler for GPIOTE events (button presses).
#[interrupt]
fn GPIOTE() {
    free(|cs| {
        if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
            let a_pressed = gpiote.channel0().is_event_triggered();
            let b_pressed = gpiote.channel1().is_event_triggered();

            let turn = match (a_pressed, b_pressed) {
                (true,  false) => Turn::Left,
                (false, true)  => Turn::Right,
                _              => Turn::None,
            };

            gpiote.channel0().reset_events();
            gpiote.channel1().reset_events();
            *TURN.borrow(cs).borrow_mut() = turn;
        }
    });
}

/// Get the next turn direction (from the most recent button press).
/// If `reset` is true, consume the turn (reset to None).
pub fn get_turn(reset: bool) -> Turn {
    free(|cs| {
        let turn = *TURN.borrow(cs).borrow();
        if reset {
            *TURN.borrow(cs).borrow_mut() = Turn::None;
        }
        turn
    })
}
