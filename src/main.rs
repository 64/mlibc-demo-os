#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(extern_types)]
#![feature(slice_take)]

extern crate alloc;

use core::{fmt::Write, panic::PanicInfo};

use alloc::boxed::Box;
use log::{info, LevelFilter};
use logger::UartLogger;
use page_table::PageTable;
use riscv::register::{
    scause, sepc, stval,
    stvec::{self, TrapMode},
};

mod allocator;
mod logger;
mod page_table;
mod userspace;

core::arch::global_asm!(include_str!("boot.asm"));
core::arch::global_asm!(include_str!("trap_handler.asm"));

extern "C" {
    fn trap_handler() -> !;
}

#[no_mangle]
extern "C" fn kernel_main(_hart_id: u64, dtb: *const u8) {
    unsafe {
        stvec::write(trap_handler as usize, TrapMode::Direct);
    }

    logger::init(LevelFilter::Trace);
    info!("Booting mlibc-demo-os...");

    let fdt = unsafe { fdt::Fdt::from_ptr(dtb).unwrap() };
    allocator::init(&fdt);

    let mut root_pt = Box::new(PageTable::new());
    root_pt.map_higher_half();

    let satp = (8 << 60) | (&*root_pt as *const PageTable as usize >> 12);
    riscv::register::satp::write(satp);
    logger::paging_initialised();

    userspace::init(&mut root_pt);

    info!("Halting...");
    exit();
}

fn exit() -> ! {
    sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::Shutdown,
        sbi::system_reset::ResetReason::NoReason,
    )
    .unwrap_or_else(|_| loop {});
    unreachable!()
}

#[panic_handler]
fn abort(info: &PanicInfo) -> ! {
    let _ = writeln!(UartLogger, "\x1b[31mKERNEL PANIC:\x1b[0m {info}");
    exit();
}

#[repr(C)]
struct TrapFrame {
    sp: u64,
    a0: u64,
}

#[no_mangle]
extern "C" fn rust_trap_handler(frame: &mut TrapFrame) {
    let scause = scause::read();
    if scause.is_exception() && scause.code() == 8 {
        userspace::handle_ecall(frame);
    } else {
        let _ = writeln!(
            UartLogger,
            concat!(
                "---------------------------\n",
                "[\x1b[31mKERNEL TRAP\x1b[0m] \x1b[31m{:?}\x1b[0m with stval = {:#x}, sepc = {:#x}",
            ),
            scause.cause(),
            stval::read(),
            sepc::read(),
        );

        let _ = sbi::system_reset::system_reset(
            sbi::system_reset::ResetType::Shutdown,
            sbi::system_reset::ResetReason::NoReason,
        );

        loop {}
    }
}
