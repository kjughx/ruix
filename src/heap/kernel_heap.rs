use super::{Addr, Heap};
use crate::global::global;

const KERNEL_HEAP_SIZE: usize = 100 * 1024 * 1024; // 100MB
const KERNEL_HEAP_START: usize = 0x01000000;
const KERNEL_ENTRIES_START: usize = 0x00007E00;

global! {
    KernelHeap,
    Heap,
    Heap::new(KERNEL_ENTRIES_START, KERNEL_HEAP_SIZE, KERNEL_HEAP_START),
    "KERNEL_HEAP"
}

pub fn alloc<T>(size: usize) -> *mut T {
    let heap = KernelHeap::get_mut();

    heap.with_wlock(|heap| -> *mut T { heap.alloc_blocks(Heap::align_block(size)).cast() })
}

pub fn realloc(old: Addr, size: usize) -> Addr {
    let heap = KernelHeap::get_mut();

    heap.with_wlock(|heap| -> Addr {
        let count = Heap::align_block(size);
        let new = heap.alloc_blocks(count);
        let src = heap.addr_to_block(new);
        let dst = heap.addr_to_block(old);

        heap.copy_blocks(src, dst, count);

        new
    })
}

pub fn free<T: ?Sized>(ptr: *mut T) {
    let heap = KernelHeap::get_mut();

    heap.with_wlock(|heap| {
        let start_block = heap.addr_to_block(ptr.cast());
        heap.mark_blocks_free(start_block);
    })
}
