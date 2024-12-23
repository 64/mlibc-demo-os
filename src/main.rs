#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(extern_types)]

extern crate alloc;

use core::{
    fmt::Write,
    panic::PanicInfo,
};

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

core::arch::global_asm!(include_str!("boot.asm"));

#[no_mangle]
extern "C" fn kernel_main(_hart_id: u64, dtb: *const u8) {
    unsafe {
        stvec::write(trap_handler as usize, TrapMode::Direct);
    }

    logger::init(LevelFilter::Trace);
    info!("Booting mlibc-demo-os...");

    let fdt = unsafe { fdt::Fdt::from_ptr(dtb).unwrap() };
    allocator::init(&fdt);

    let mut root_pt = PageTable::new();
    root_pt.map_higher_half();

    let satp = (8 << 60) | (&root_pt as *const PageTable as usize >> 12);
    riscv::register::satp::write(satp);
    logger::paging_initialised();

    info!("Paging initialised.");

    root_pt.map_page(0x40000, page_table::READ);

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
