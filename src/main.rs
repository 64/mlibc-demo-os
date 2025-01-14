#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(extern_types)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate alloc;

use core::{fmt::Write, panic::PanicInfo};

use alloc::boxed::Box;
use log::{info, LevelFilter};
use logger::UartLogger;
use page_table::PageTable;
use riscv::register::stvec::{self, TrapMode};

mod allocator;
mod logger;
mod page_table;
mod syscalls;
mod trap;
mod userspace;

core::arch::global_asm!(include_str!("boot.asm"));
core::arch::global_asm!(include_str!("trap_handler.asm"));

extern "C" {
    fn trap_handler() -> !;
}

#[no_mangle]
extern "C" fn kernel_main(_hart_id: u64, dtb: *const u8) -> ! {
    unsafe {
        stvec::write(trap_handler as usize, TrapMode::Direct);
    }

    logger::init(LevelFilter::Info);
    info!("Booting mlibc-demo-os...");

    let fdt = unsafe { fdt::Fdt::from_ptr(dtb).unwrap() };
    allocator::init(&fdt);

    let mut root_pt = Box::new(PageTable::new());
    root_pt.init_root_table();

    // Enable Sv39 paging and set the root PT.
    let satp = (8 << 60) | (&*root_pt as *const PageTable as usize >> 12);
    riscv::register::satp::write(satp);
    logger::paging_initialised();

    userspace::init(&mut root_pt)
}

pub fn exit() -> ! {
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
