use core::{alloc::GlobalAlloc, ptr::null_mut};

use fdt::Fdt;
use log::{debug, trace};

const MAX_SUPPORTED_ALIGN: usize = 4096;

struct BumpAllocator {
    region: &'static mut [u8],
    pos: usize,
}

impl BumpAllocator {
    pub fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        assert!(align <= MAX_SUPPORTED_ALIGN);
        debug_assert!(align > 0);
        let align_mask = !(align - 1);
        let alloc_start = (self.pos + align - 1) & align_mask;
        let alloc_end = alloc_start + size;
        if alloc_end > self.region.len() {
            return null_mut();
        }

        self.pos = alloc_end;
        let out = self.region.as_mut_ptr().wrapping_add(alloc_start);
        trace!("Allocated {} bytes at 0x{:x}", size, out as usize);
        out
    }
}

struct GlobalBumpAllocator {
    bump: spin::Mutex<Option<BumpAllocator>>,
}

#[global_allocator]
static ALLOCATOR: GlobalBumpAllocator = GlobalBumpAllocator {
    bump: spin::Mutex::new(None),
};

unsafe impl GlobalAlloc for GlobalBumpAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.bump.lock().as_mut().unwrap().alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

extern "C" {
    type Symbol;
    static __kernel_start_phys: Symbol;
    static __kernel_end_phys: Symbol;
}

pub fn init(fdt: &Fdt) {
    let mem = fdt.memory().regions().next().unwrap();
    let mut start_addr = mem.starting_address as usize;
    let end_addr = start_addr + mem.size.unwrap();

    // Make sure we don't allocate over memory that's already used by the kernel.
    let kernel_end_addr = unsafe { &__kernel_end_phys as *const Symbol as usize };
    start_addr = start_addr.max(kernel_end_addr);

    assert_eq!(start_addr % 0x1000, 0);
    debug!(
        "Initialised allocator: {:#x}, {} MiB",
        start_addr,
        (end_addr - start_addr) / 1024 / 1024
    );

    let region =
        unsafe { core::slice::from_raw_parts_mut(start_addr as *mut u8, end_addr - start_addr) };

    *ALLOCATOR.bump.lock() = Some(BumpAllocator { region, pos: 0 });
}
