mod x86;
pub use x86::paging::Paging;
pub use x86::process::Process;
pub use x86::X86 as Platform;

use crate::sync::Shared;

// Never use this as a Trait object
// This just outlines the methods that a platform is expected to implement
trait Platform_
where
    Self: Sized,
{
    /// Initialize whatever needs initializing
    fn init() {}
}

trait Paging_ {
    unsafe fn enable(&self);
    unsafe fn disable(&self);
    unsafe fn switch(&self);
    fn map(&mut self, vaddr: usize, paddr: usize, flags: usize);
}

trait Process_
where
    Self: Sized,
{
    fn new(filename: &str) -> Result<Shared<Self>, usize>;
    fn exec_new(filename: &str) -> Result<(), usize>;
    fn exec(this: Shared<Self>);
}
