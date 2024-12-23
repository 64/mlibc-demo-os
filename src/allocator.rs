use core::{alloc::GlobalAlloc, cell::UnsafeCell, ptr::null_mut};

const MAX_SUPPORTED_ALIGN: usize = 4096;

struct BumpAllocator {
    region_start: *mut u8,
    region_size: usize,
    pos: UnsafeCell<usize>,
}

unsafe impl Sync for BumpAllocator {}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    region_start: null_mut(),
    region_size: 0,
    pos: UnsafeCell::new(0),
};

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        if align > MAX_SUPPORTED_ALIGN {
            return null_mut();
        }

        let align_mask = !(align - 1);
        let alloc_start = (unsafe { *self.pos.get() } + align) & align_mask;
        let alloc_end = alloc_start + size;
        if alloc_end > self.region_size {
            return null_mut();
        }

        unsafe {
            *self.pos.get() = alloc_end;
        }
        self.region_start.add(alloc_start)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}
