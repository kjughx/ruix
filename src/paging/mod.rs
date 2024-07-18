use crate::{lock, sync::Global, Error};

pub mod pagedirectory;
pub mod pagetable;

use pagedirectory::PageDirectory;

static mut KERNEL_DIRECTORY: Global<PageDirectory> = Global::new(
    || PageDirectory::new(PAGE_IS_WRITABLE | PAGE_IS_PRESENT | PAGE_ACCESS_ALL),
    "KERNEL_PAGE_DIRECTORY",
);

pub struct KernelPage;
impl KernelPage {
    pub fn get() -> PageDirectory {
        let directory = unsafe { lock!(KERNEL_DIRECTORY) };
        directory.inner().clone()
    }

    pub fn load() {
        let kernel_directory = unsafe { lock!(KERNEL_DIRECTORY) };
        kernel_directory.load();
    }

    pub fn map(vaddr: Addr, paddr: Addr, flags: Flags) -> Result<(), Error> {
        let mut directory = unsafe { lock!(KERNEL_DIRECTORY) };
        directory.map(vaddr, paddr, flags)
    }
}

static mut CURRENT_DIRECTORY: Global<PageDirectory> =
    Global::new(|| PageDirectory::new(0), "CURRENT_DIRECTORY");

pub struct Paging;
impl Paging {
    pub fn enable() {
        unsafe {
            core::arch::asm!(
                r#"
                mov eax, cr0
                or eax, 0x80000000
                mov cr0, eax
                "#
            )
        }
    }

    pub fn switch(directory: PageDirectory) {
        directory.load();
        unsafe { *lock!(CURRENT_DIRECTORY) = directory };
    }
}

const ENTRIES_PER_TABLE: usize = 1024;
const PAGE_SIZE: usize = 4096;

#[derive(Clone, Copy)]
pub struct Addr(pub usize);
impl Addr {
    fn raw(&self) -> usize {
        self.0
    }

    fn _align(&self) -> Self {
        if self.is_aligned() {
            return *self;
        }

        Self(self.0 + (PAGE_SIZE - self.0 % PAGE_SIZE))
    }

    fn align_lower(&self) -> Self {
        if self.is_aligned() {
            return *self;
        }

        Self(self.0 - self.0 % PAGE_SIZE)
    }

    fn is_aligned(&self) -> bool {
        self.0 % PAGE_SIZE == 0
    }

    fn offset(&self, offset: usize) -> Self {
        Self(self.0 + offset)
    }

    fn as_offset(&self) -> Offset {
        Offset((self.0 % (ENTRIES_PER_TABLE * PAGE_SIZE)) / PAGE_SIZE)
    }

    fn as_page(&self) -> Page {
        Page(self.0 / (ENTRIES_PER_TABLE * PAGE_SIZE))
    }
}

pub struct Page(pub usize);
pub struct Offset(pub usize);

type Flags = u16;
pub const PAGE_IS_PRESENT: Flags = 1 << 0;
pub const PAGE_IS_WRITABLE: Flags = 1 << 1;
pub const PAGE_ACCESS_ALL: Flags = 1 << 2;
pub const PAGE_WRITE_THROUGH: Flags = 1 << 3;
pub const PAGE_CACHE_DISABLED: Flags = 1 << 4;
