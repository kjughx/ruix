use core::marker::PhantomData;

use super::{Addr, Flags, Offset, ENTRIES_PER_TABLE, PAGE_SIZE};

#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry(usize);
#[derive(Debug, Clone, Copy)]
pub struct PageTable {
    pub entries: *mut PageTableEntry,
    _marker: PhantomData<[PageTableEntry]>,
}

pub const ENTRY_SIZE: usize = core::mem::size_of::<PageTableEntry>();
pub const TABLE_SIZE: usize = ENTRIES_PER_TABLE * ENTRY_SIZE;

impl PageTableEntry {
    pub fn new(addr: Addr, flags: Flags) -> Self {
        Self(addr.raw() | flags as usize)
    }

    pub fn addr(&self) -> Addr {
        Addr(self.0 & 0xfffff000)
    }

    pub fn flags(&self) -> Flags {
        (self.0 & 0x00000fff) as Flags
    }
}

impl PageTable {
    pub fn new(offset: Offset, flags: Flags) -> Self {
        let entries: *mut PageTableEntry = alloc!(ENTRIES_PER_TABLE * ENTRY_SIZE);
        for entry in 0..ENTRIES_PER_TABLE {
            let addr = Addr(offset.0 + entry * PAGE_SIZE);
            unsafe {
                entries.add(entry).write(PageTableEntry::new(addr, flags));
            }
        }

        Self {
            entries: (entries as usize | flags as usize) as *mut PageTableEntry,
            _marker: PhantomData,
        }
    }

    pub fn free(self) {
        free!(self.entries);
    }

    /// # Safety
    pub unsafe fn from_ptr(ptr: *mut PageTable) -> Self {
        let table_with_flags = unsafe { *ptr };
        let table_ptr = table_with_flags.entries as usize & 0xfffff000;
        Self {
            entries: table_ptr as *mut PageTableEntry,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, offset: Offset) -> PageTableEntry {
        unsafe { *self.entries.add(offset.0) }
    }

    pub fn set(&mut self, offset: Offset, entry: PageTableEntry) {
        unsafe { self.entries.add(offset.0).write(entry) }
    }
}
