#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use omwei_iot::{decode, encode, SemanticAtom, AMBIENT_TEMPERATURE_ID};
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Example application path: construct a canonical temperature atom,
    // serialize it, and decode it again without allocating or using std.
    let result = SemanticAtom::new(AMBIENT_TEMPERATURE_ID, 22_500)
        .map(encode)
        .and_then(|wire| decode(&wire));

    // Keep the result observable to the optimizer. A board-specific firmware
    // can replace this loop with its sensor, transport, and error handling.
    loop {
        core::hint::black_box(&result);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
