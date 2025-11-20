use core::panic::PanicInfo;

use crate::println;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("PANIC: {info}");
    loop {}
}
