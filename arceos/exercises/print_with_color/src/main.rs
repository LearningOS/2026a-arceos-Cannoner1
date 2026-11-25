#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;


use axstd::os::arceos::modules::axlog::*;
//with_color!(34,"[WithColor]: Hello, Arceos!");

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    ax_println!("\u{1B}[{}m{}\u{1B}[m",34,"[WithColor]: Hello, Arceos!\n");
}
