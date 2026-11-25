#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // ANSI 转义序列：32 = 绿色，0 = 恢复默认颜色。
    println!("\x1b[32mHello, Arceos!\x1b[0m");
}
