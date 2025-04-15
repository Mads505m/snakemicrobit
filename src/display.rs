use core::cell::RefCell;
use cortex_m::interrupt::{Mutex, free};
use microbit::display::nonblocking::Display;
use microbit::gpio::DisplayPins;
use microbit::pac::{self, TIMER1, interrupt};
use tiny_led_matrix::Render;

static DISPLAY: Mutex<RefCell<Option<Display<TIMER1>>>> = Mutex::new(RefCell::new(None));

/// Initialize the non-blocking display (uses TIMER1 for refresh).
pub(crate) fn init_display(board_timer: TIMER1, board_display: DisplayPins) {
    let display = Display::new(board_timer, board_display);
    free(move |cs| {
        *DISPLAY.borrow(cs).borrow_mut() = Some(display);
    });
    unsafe { pac::NVIC::unmask(pac::Interrupt::TIMER1); }
}

/// Show an image (implementing the `Render` trait) on the LED matrix.
pub(crate) fn display_image(image: &impl Render) {
    free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.show(image);
        }
    })
}

/// Clear the LED matrix (turn off all LEDs).
pub(crate) fn clear_display() {
    free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.clear();
        }
    })
}

/// Interrupt handler for TIMER1 - drives the display refresh.
#[interrupt]
fn TIMER1() {
    free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.handle_display_event();
        }
    });
}
