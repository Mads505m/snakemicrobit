#![no_std]
#![no_main]
#![warn(unsafe_code)]

mod game;
mod controls;
mod display;

use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use cortex_m::asm;

use microbit::{
    board::Board,
    display::nonblocking::{BitImage, GreyscaleImage},
    hal::{prelude::*, rng::Rng, timer::Timer},
};

use rtt_target::{rtt_init_print, rprintln, rprint};

use crate::controls::{get_turn, init_buttons};
use crate::display::{clear_display, display_image, init_display};
use crate::game::{Game, GameStatus};

#[entry]
fn main() -> ! {
    rtt_init_print!(); // ✅ Setup RTT over USB
    let mut board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0).into_periodic();
    let mut rng = Rng::new(board.RNG);
    let mut game = Game::new(rng.random_u32());

    init_buttons(board.GPIOTE, board.buttons);
    init_display(board.TIMER1, board.display_pins);

    loop {
        let matrix = game.game_matrix(6, 3, 9);

        // ✅ Send matrix to browser/terminal via RTT
        for row in &matrix {
            for &val in row {
                rprint!("{}", val); // send as flat 25-char string
            }
        }
        rprintln!(""); // newline

        // Also display on micro:bit
        let image = GreyscaleImage::new(&matrix);
        display_image(&image);
        timer.delay_ms(game.step_len_ms());

        match game.status {
            GameStatus::Ongoing => game.step(get_turn(true)),
            _ => {
                for _ in 0..3 {
                    clear_display();
                    timer.delay_ms(200u32);
                    display_image(&image);
                    timer.delay_ms(200u32);
                }
                clear_display();
                display_image(&BitImage::new(&game.score_matrix()));
                timer.delay_ms(2000u32);
                break;
            }
        }
    }

    loop {
        asm::wfi();
    }
}
