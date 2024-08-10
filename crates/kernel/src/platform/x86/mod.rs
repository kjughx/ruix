use implement::implement;
use paging::{KernelPage, Paging};

use super::{Platform_, Process_};

pub mod boot;
pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod paging;
pub mod process;
mod syscall;

pub struct X86;

#[implement]
impl Platform_ for X86 {
    fn init() {
        gdt::GDT::load();
        idt::IDT::load();

        KernelPage::switch();
        Paging::enable();
    }
}
