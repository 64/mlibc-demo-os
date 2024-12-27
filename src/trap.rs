use core::fmt::Write;
use riscv::register::{scause, sepc, stval};

use crate::{logger::UartLogger, syscalls};

const ECALL_SCAUSE_CODE: usize = 8;

// Layout defined in trap_handler.asm.
#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame {
    pub sp: u64,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
    pub a4: u64,
    pub a5: u64,
    pub a6: u64,
    pub a7: u64,
    pub t0: u64,
    pub t1: u64,
    pub t2: u64,
    pub t3: u64,
    pub t4: u64,
    pub t5: u64,
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub s8: u64,
    pub s9: u64,
    pub s10: u64,
    pub s11: u64,
    pub ra: u64,
    pub tp: u64,
    pub gp: u64,
    pub t6: u64,
}

#[no_mangle]
extern "C" fn rust_trap_handler(frame: &mut TrapFrame) {
    let scause = scause::read();
    if scause.is_exception() && scause.code() == ECALL_SCAUSE_CODE {
        syscalls::handle_syscall(frame);
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
