#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]

extern crate alloc;

use core::{fmt::Write, panic::PanicInfo};

use log::{info, LevelFilter};
use logger::UartLogger;
use riscv::register::{
    scause, sepc, stval,
    stvec::{self, TrapMode},
};

mod logger;
mod page_table;
mod allocator;

core::arch::global_asm!(include_str!("boot.asm"));

#[no_mangle]
extern "C" fn kernel_main(_hart_id: u64, _dtb: *const u8) {
    unsafe {
        stvec::write(trap_handler as usize, TrapMode::Direct);
    }

    logger::init(LevelFilter::Trace);
    info!("Booting mlibc-demo-os...");

    loop {}
}

#[panic_handler]
fn abort(info: &PanicInfo) -> ! {
    let _ = writeln!(UartLogger, "\x1b[31mKERNEL PANIC:\x1b[0m {info}");

    sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::Shutdown,
        sbi::system_reset::ResetReason::NoReason,
    )
    .unwrap_or_else(|_| loop {});

    unreachable!()
}

#[repr(align(4))]
fn trap_handler() {
    let _ = writeln!(
        UartLogger,
        concat!(
            "---------------------------\n",
            "[\x1b[31mKERNEL TRAP\x1b[0m] \x1b[31m{:?}\x1b[0m with stval = {:#x}, sepc = {:#x}",
        ),
        scause::read().cause(),
        stval::read(),
        sepc::read(),
    );

    let _ = sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::Shutdown,
        sbi::system_reset::ResetReason::NoReason,
    );

    loop {}
}
