use core::ptr::{self, Unique};

mod kernel_heap;
pub use kernel_heap::{alloc, free, realloc};

pub const HEAP_BLOCK_SIZE: usize = 4096;

const BLOCK_FREE: u8 = 0;
const BLOCK_TAKEN: u8 = 1 << 0;
const BLOCK_FIRST: u8 = 1 << 1;
const BLOCK_HAS_NEXT: u8 = 1 << 2;

#[derive(Debug)]
enum MemoryError {
    OutOfMemory,
}

pub type Addr = *mut u8;

pub struct Heap {
    entries: Unique<[u8]>,
    count: usize,
    start: Addr,
}

impl Heap {
    pub fn new(entries: usize, size: usize, start: usize) -> Self {
        let real_start = start + start % HEAP_BLOCK_SIZE;

        let entries = unsafe {
            Unique::new_unchecked(ptr::slice_from_raw_parts_mut(
                entries as *mut u8,
                size / HEAP_BLOCK_SIZE,
            ))
        };

        Self {
            entries,
            count: size / HEAP_BLOCK_SIZE,
            start: real_start as Addr,
        }
    }

    fn block_to_addr(&self, block: usize) -> Addr {
        (self.start as usize + block * HEAP_BLOCK_SIZE) as Addr
    }
    fn addr_to_block(&self, addr: Addr) -> usize {
        debug_assert!(addr >= self.start, "{addr:#?}");
        (addr as usize - self.start as usize) / (HEAP_BLOCK_SIZE)
    }

    fn mark_blocks_taken(&mut self, start_block: usize, total_blocks: usize) {
        if total_blocks == 1 {
            unsafe {
                self.entries.as_mut()[start_block] = BLOCK_TAKEN | BLOCK_FIRST;
            }
            return;
        }

        unsafe {
            self.entries.as_mut()[start_block] = BLOCK_TAKEN | BLOCK_FIRST | BLOCK_HAS_NEXT;
            ptr::write_bytes(
                (self.entries.as_ptr() as *mut u8).offset(start_block as isize + 1),
                BLOCK_TAKEN | BLOCK_HAS_NEXT,
                total_blocks - 2,
            );
            self.entries.as_mut()[start_block + total_blocks - 1] = BLOCK_TAKEN | BLOCK_FIRST;
        }
    }

    fn mark_blocks_free(&mut self, start_block: usize) {
        for i in start_block..self.count {
            let entry = unsafe {
                let _entry = self.entries.as_ref()[i];
                self.entries.as_mut()[i] = BLOCK_FREE;
                _entry
            };

            if entry & BLOCK_HAS_NEXT == 0 {
                break;
            }
        }
    }

    fn entry_type(entries: Unique<[u8]>, offset: usize) -> u8 {
        unsafe { entries.as_ref()[offset] & 0x0f }
    }

    fn get_free_block(&self, count: usize) -> Result<usize, MemoryError> {
        let mut bc = 0;
        let mut bs: isize = -1;

        for i in 0..self.count {
            if Self::entry_type(self.entries, i) != BLOCK_FREE {
                bc = 0;
                bs = -1;
                continue;
            }

            if bs == -1 {
                bs = i as isize;
            }

            bc += 1;
            if bc == count {
                break;
            }
        }
        if bs == -1 {
            return Err(MemoryError::OutOfMemory);
        }

        Ok(bs as usize)
    }

    fn alloc_blocks(&mut self, block_count: usize) -> *mut u8 {
        let start_block = self.get_free_block(block_count).unwrap();

        let addr = self.block_to_addr(start_block);
        self.mark_blocks_taken(start_block, block_count);

        addr
    }

    fn copy_block(&mut self, src: usize, dst: usize) {
        let src = self.block_to_addr(src);
        let dst = self.block_to_addr(dst);

        for i in 0..HEAP_BLOCK_SIZE as isize {
            unsafe { *dst.offset(i) = *src.offset(i) }
        }
    }

    fn copy_blocks(&mut self, src: usize, dst: usize, count: usize) {
        for i in 0..count {
            self.copy_block(src + i, dst + i)
        }
    }

    fn align_block(val: usize) -> usize {
        if val < HEAP_BLOCK_SIZE {
            1
        } else if val % HEAP_BLOCK_SIZE == 0 {
            return val / HEAP_BLOCK_SIZE;
        } else {
            return val / HEAP_BLOCK_SIZE + 1;
        }
    }
}
