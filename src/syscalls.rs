use alloc::boxed::Box;
use log::{debug, info, trace};
use riscv::register::{sepc, sstatus};

use crate::{logger::UartLogger, TrapFrame};

#[derive(Debug)]
enum Syscall {
    Exit,
    Write,
}

impl From<u64> for Syscall {
    fn from(val: u64) -> Syscall {
        match val {
            x if x == Syscall::Write as u64 => Syscall::Write,
            _ => Syscall::Exit,
        }
    }
}

fn copy_from_user(uptr: u64, size: usize) -> Box<[u8]> {
    trace!("copy_from_user: {:#x}, size = {}", uptr, size);
    let mut out: Box<[u8]> = vec![0; size].into_boxed_slice();

    unsafe {
        sstatus::set_sum();
        core::ptr::copy_nonoverlapping(uptr as *mut u8, out.as_mut_ptr(), size);
        sstatus::clear_sum();
    }

    out
}

pub fn handle_syscall(frame: &mut TrapFrame) {
    let pc = sepc::read();
    let syscall = Syscall::from(frame.a0);
    info!("Handling Syscall::{:?} at pc {:#x}", syscall, pc);

    let ret = match syscall {
        Syscall::Exit => {
            info!("Userspace program exited with status code {}", frame.a1);
            crate::exit();
        }
        Syscall::Write => {
            let fd = frame.a1;
            let buf = frame.a2;
            let size = frame.a3;

            assert_eq!(fd, 1);

            let data = copy_from_user(buf, size as usize);
            for c in data {
                UartLogger.write(c);
            }

            size
        }
    };

    // sepc points to the ecall instruction; prepare returning to the next one.
    sepc::write(pc + 4);

    frame.a0 = ret;
    debug!("...returned {}", ret);
}
