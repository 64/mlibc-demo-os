use core::fmt::Write;
use log::{debug, info, trace};
use riscv::register::sepc;

use crate::{
    logger::UartLogger,
    page_table::{READ, USER, VALID, WRITE},
    trap::TrapFrame,
    userspace::{self, copy_from_user},
};

const MLIBC_LOG_MAGIC_FD: u64 = 2;

#[derive(Debug)]
enum Syscall {
    Exit,
    Write,
    Mmap,
}

impl From<u64> for Syscall {
    fn from(val: u64) -> Syscall {
        match val {
            x if x == Syscall::Write as u64 => Syscall::Write,
            x if x == Syscall::Mmap as u64 => Syscall::Mmap,
            x if x == Syscall::Exit as u64 => Syscall::Exit,
            _ => panic!("unknown syscall number {val}"),
        }
    }
}

pub fn handle_syscall(frame: &mut TrapFrame) {
    let pc = sepc::read();
    let syscall = Syscall::from(frame.a7);
    debug!(
        "Handling Syscall::{:?} at pc {:#x}: a0 = {:#x}, a1 = {:#x}, a2 = {:#x}",
        syscall, pc, frame.a0, frame.a1, frame.a2
    );

    let ret = match syscall {
        Syscall::Exit => {
            info!(
                "Userspace program exited with status code {}",
                frame.a0 as i64
            );
            crate::exit();
        }
        Syscall::Write => {
            let fd = frame.a0;
            let buf = frame.a1;
            let size = frame.a2;

            if fd == MLIBC_LOG_MAGIC_FD {
                write!(UartLogger, "mlibc: ").unwrap();
            }

            let data = unsafe { copy_from_user(buf, size as usize) };
            for c in data {
                UartLogger.write(c);
            }

            if fd == MLIBC_LOG_MAGIC_FD {
                UartLogger.write(b'\n');
            }

            size
        }
        Syscall::Mmap => {
            let addr = frame.a0;
            let len = frame.a1;
            let _prot = frame.a2;
            let flags = frame.a3;
            let _fd = frame.a4;
            let _offset = frame.a5;

            assert_eq!(addr, 0);
            assert_eq!(flags, 32 | 2 /* MAP_ANONYMOUS | MAP_PRIVATE */);

            let mut task_lock = userspace::TASK.lock();
            let task = task_lock.as_mut().unwrap();

            // Pick a suitable address. We'll just bump a pointer to keep track of where we
            // should allocate.
            let addr = 512 * 1024 * 1024 + task.heap_pages_allocated * 0x1000;
            let num_pages = len.div_ceil(0x1000);
            task.heap_pages_allocated += num_pages;

            let pt = unsafe { &mut *task.pt };

            for virt in (addr..addr + (num_pages - 1) * 0x1000).step_by(0x1000) {
                // todo: prot
                pt.map_page(virt, VALID | USER | READ | WRITE);
            }

            trace!("mmap allocated {} pages from {:#x}", num_pages, addr);
            addr as u64
        }
    };

    // sepc points to the ecall instruction; prepare returning to the one after.
    sepc::write(pc + 4);

    frame.a0 = ret;
    debug!("...returned {:#x}", ret);
}
