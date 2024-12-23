use core::{arch::asm, cmp, ptr};

use log::info;
use riscv::register::{sepc, sstatus::SPP};
use xmas_elf::{
    program::{SegmentData, Type},
    ElfFile,
};

use crate::{page_table::{PageTable, EXECUTE, READ, USER, VALID, WRITE}, TrapFrame};

const USERSPACE_BINARY: &[u8] = include_bytes!("../target/riscv64imac-unknown-none-elf/user_test");

extern "C" {
    fn ret_to_user(stack: u64) -> !;
}
core::arch::global_asm!(include_str!("ret_to_user.asm"));

pub fn init(root: &mut PageTable) {
    info!("Loading userspace program...");

    let entry = load_elf(root);

    let stack = map_stack(root);

    unsafe {
        riscv::register::sstatus::set_spp(SPP::User);
        riscv::register::sepc::write(entry as usize);
        ret_to_user(stack);
    }
}

pub fn handle_ecall(frame: &mut TrapFrame) {
    let sepc = sepc::read();
    info!("Handling syscall {} (sepc = {:x})", frame.a0, sepc);

    // sepc points to the ecall instruction; prepare returning to the next one.
    sepc::write(sepc + 4);
}

pub fn load_elf(root: &mut PageTable) -> u64 {
    let elf = ElfFile::new(USERSPACE_BINARY).unwrap();
    for phdr in elf
        .program_iter()
        .filter(|phdr| phdr.get_type() == Ok(Type::Load))
    {
        // trace!("{phdr}");

        let base = phdr.virtual_addr();
        let mut data = match phdr.get_data(&elf) {
            Ok(SegmentData::Undefined(data)) => data,
            _ => panic!("no segment data"),
        };

        for virt in (base..base + phdr.mem_size()).step_by(0x1000) {
            let mut prot = 0;
            if phdr.flags().is_read() {
                prot |= READ;
            }
            if phdr.flags().is_write() {
                prot |= WRITE;
            }
            if phdr.flags().is_execute() {
                prot |= EXECUTE;
            }

            let data_to_copy = data.take(..cmp::min(data.len(), 0x1000)).unwrap();
            let addr = root.map_page(virt, VALID | USER | prot);

            unsafe {
                ptr::copy_nonoverlapping(data_to_copy.as_ptr(), addr, data_to_copy.len());
            }
        }
    }

    elf.header.pt2.entry_point()
}

pub fn map_stack(root: &mut PageTable) -> u64 {
    let stack_start = 0xF000000;
    let stack_pages = 16;

    for i in 0..stack_pages {
        root.map_page(stack_start + i * 0x1000, VALID | READ | WRITE);
    }

    stack_start
}
