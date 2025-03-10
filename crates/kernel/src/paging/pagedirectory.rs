use core::marker::PhantomData;
use core::ops::Range;

use crate::{trace, traceln};

use super::{
    pagetable::{PageTable, PageTableEntry, ENTRY_SIZE},
    Addr, Flags, Offset, Page, ENTRIES_PER_TABLE, PAGE_SIZE,
};

#[derive(Clone, Copy)]
pub struct PageDirectory {
    tables: *mut PageTable,
    _marker: PhantomData<[PageTable]>,
}

impl PageDirectory {
    pub fn new(pte_flags: Flags) -> Self {
        // let tables = alloc::<PageTable>(ENTRIES_PER_TABLE * ENTRY_SIZE);
        let tables: *mut PageTable = alloc!(ENTRIES_PER_TABLE * ENTRY_SIZE);

        for dentry in 0..ENTRIES_PER_TABLE {
            unsafe {
                tables.add(dentry).write(PageTable::new(
                    Offset(dentry * ENTRIES_PER_TABLE * PAGE_SIZE),
                    pte_flags,
                ));
            }
        }

        Self {
            tables,
            _marker: PhantomData,
        }
    }

    pub fn ptr(&self) -> *mut PageTable {
        self.tables
    }

    pub fn free(&mut self) {
        for dentry in 0..ENTRIES_PER_TABLE {
            let table = self.get_table(Page(dentry));
            table.free();
        }
    }

    pub fn get_entry(&self, vaddr: Addr) -> PageTableEntry {
        let ptr = self.ptr() as usize + ((vaddr.0 >> 12) & 0x3FF) * 4;

        PageTableEntry::new(Addr(ptr & 0xfffff000), (ptr & 0x00000fff) as u16)
    }

    pub fn inspect(&self, drange: Range<usize>, trange: Range<usize>) {
        traceln!("Page Directory");
        for i in drange {
            unsafe {
                trace!(
                    "\n\nPage {} 0x{:x}\n\n",
                    i,
                    (*self.tables.add(i)).entries as usize
                );
            };
            let page = self.get_table(Page(i));
            let mut k = 0;
            for j in trange.clone() {
                let entry = page.get(Offset(j));
                trace!("0x{:x} - 0b{:b}\t", entry.addr().0, entry.flags());
                k += 1;
                if k % 5 == 0 {
                    trace!("\n");
                    k = 0;
                }
            }
        }
    }

    fn _free(self) {
        for i in 0..ENTRIES_PER_TABLE {
            let table = self.get_table(Page(i));
            table.free()
        }

        free!(self.tables)
    }

    fn get_table(&self, page: Page) -> PageTable {
        unsafe { PageTable::from_ptr(self.tables.add(page.0)) }
    }

    fn set(&mut self, vaddr: Addr, entry: PageTableEntry) {
        assert!(vaddr.is_aligned());

        let dentry = vaddr.as_page();
        let mut table = self.get_table(dentry);

        let tentry = vaddr.as_offset();

        table.set(tentry, entry)
    }

    pub fn map(&mut self, vaddr: Addr, paddr: Addr, flags: Flags) {
        assert!(vaddr.is_aligned(), "Invalid virtual address: {vaddr}");
        assert!(paddr.is_aligned(), "Invalid physical address: {paddr}");

        self.set(vaddr, PageTableEntry::new(paddr, flags));
    }

    pub fn map_range(&mut self, vstart: Addr, pstart: Addr, pend: Addr, flags: Flags) {
        assert!(pend.raw() >= pstart.raw(), "Invalid address range");

        let count = (pend.raw() - pstart.raw()) / PAGE_SIZE;
        for page in 0..count {
            self.map(
                vstart.offset(page * PAGE_SIZE),
                pstart.offset(page * PAGE_SIZE),
                flags,
            );
        }
    }

    pub fn get_paddr(&self, vaddr: Addr) -> Addr {
        self.get_table(vaddr.align_lower().as_page())
            .get(vaddr.as_offset())
            .addr()
    }

    pub fn get_flags(&self, vaddr: Addr) -> Flags {
        self.get_table(vaddr.align_lower().as_page())
            .get(vaddr.as_offset())
            .flags()
    }
}
