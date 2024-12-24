use core::alloc::{Allocator, Layout};

use alloc::{alloc::Global, boxed::Box};

#[repr(align(4096))]
pub struct PageTable {
    entries: [u64; 512],
}

pub const VALID: u64 = 1;
pub const READ: u64 = 1 << 1;
pub const WRITE: u64 = 1 << 2;
pub const EXECUTE: u64 = 1 << 3;
pub const USER: u64 = 1 << 4;
pub const GLOBAL: u64 = 1 << 5;

pub const PTE_PPN_MASK: u64 = 0xff_ffff_ffff_fc00;
pub const USEFUL_FLAGS_MASK: u64 = 0x3f;

impl PageTable {
    pub fn new() -> Self {
        Self { entries: [0; 512] }
    }

    // For the root page table only.
    pub fn map_higher_half(&mut self) {
        for (i, entry) in self.entries.iter_mut().enumerate() {
            if i == 0 {
                // Leave the first 1GiB unmapped for userspace.
                continue;
            } else if i == 511 {
                // Map the zero 1GiB page at 511GiB. Useful for MMIO (UART).
                *entry = READ | WRITE | EXECUTE | GLOBAL | VALID;
            } else {
                *entry = (i as u64) << 28 | READ | WRITE | EXECUTE | GLOBAL | VALID;
            }
        }
        riscv::asm::sfence_vma_all();
    }

    pub fn map_page(&mut self, virt: u64, flags: u64) -> *mut u8 {
        assert!(flags & (READ | WRITE | EXECUTE) != 0);

        let out = self.do_map(virt & !0xfff, (flags | VALID) & USEFUL_FLAGS_MASK, 2);
        riscv::asm::sfence_vma_all();

        // debug!(
        //     "Mapped virtual address {:#x} -> physical address {:#x} with flags
        // {:#b}",     virt, out as u64, flags
        // );
        out
    }

    fn do_map(&mut self, virt: u64, flags: u64, depth: usize) -> *mut u8 {
        let vpn = [
            (virt >> 12) & 0x1ff,
            (virt >> 21) & 0x1ff,
            (virt >> 30) & 0x1ff,
        ];

        let entry = &mut self.entries[vpn[depth] as usize];

        // Leaf, just allocate an empty page and return.
        if depth == 0 {
            assert!(*entry & VALID == 0);

            let mut page = Global
                .allocate(Layout::from_size_align(0x1000, 0x100).unwrap())
                .unwrap();
            unsafe {
                page.as_mut().fill(0);
            }

            let page_addr = page.as_ptr().as_mut_ptr() as u64;
            *entry = (page_addr >> 2 & PTE_PPN_MASK) | VALID | flags;
            return page_addr as *mut u8;
        }

        // Allocate a new intermediate page table if necessary.
        if *entry & VALID == 0 {
            let pt = Box::new(PageTable::new());
            let target = Box::leak(pt) as *mut PageTable as u64;
            *entry = (target >> 2 & PTE_PPN_MASK) | VALID;
        }

        // Extract the address of the next level of PT we need to modify.
        let addr = (*entry & PTE_PPN_MASK) << 2;
        let page_table = unsafe { &mut *(addr as *mut PageTable) };
        assert!(!core::ptr::eq(self, page_table));

        page_table.do_map(virt, flags, depth - 1)
    }
}
