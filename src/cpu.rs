use core::arch::asm;
use core::fmt::Display;

use crate::paging::Paging;
use crate::sync::Shared;
use crate::task::Task;

use crate::packed::{packed, Packed};

const PROGRAM_VIRTUAL_ADDRESS: usize = 0x400000;
const USER_DATA_SEGMENT: usize = 0x23;
const USER_CODE_SEGMENT: usize = 0x1B;
const PROGRAM_VIRTUAL_STACK_START: usize = 0x3FF000;

#[allow(dead_code)]

pub type Registers = InterruptFrame;

impl Registers {
    pub fn user_default() -> Self {
        Self {
            edi: 0,
            esi: 0,
            ebp: 0,
            ebx: 0,
            edx: 0,
            ecx: 0,
            eax: 0,
            unused: 0,

            errno: 0,
            ip: PROGRAM_VIRTUAL_ADDRESS,
            cs: USER_CODE_SEGMENT,
            flags: 0,
            sp: PROGRAM_VIRTUAL_STACK_START,
            ss: USER_DATA_SEGMENT,
        }
    }

    pub fn save(&mut self, frame: InterruptFrame) {
        self.edi = frame.edi;
        self.esi = frame.esi;
        self.ebp = frame.ebp;
        self.ebx = frame.ebx;
        self.edx = frame.edx;
        self.ecx = frame.ecx;
        self.eax = frame.eax;
        self.ip = frame.ip;
        self.cs = frame.cs;
        self.flags = frame.flags;
        self.sp = frame.sp;
        self.ss = frame.ss;
    }

    /// # Safety
    // This fucks with registers
    #[naked]
    #[no_mangle]
    pub unsafe extern "C" fn restore(regs: *const Registers) {
        asm!(
            r#"
            push ebp
            mov ebp, esp
            mov ebx, [ebp+8] // ebx = regs;
            mov edi, [ebx]   // edi = *regs;
            mov esi, [ebx+4] // esi = *(regs + 4);
            mov ebp, [ebx+8] // ebp = *(regs + 8);
            mov edx, [ebx+16]
            mov ecx, [ebx+20]
            mov eax, [ebx+24]
            mov ebx, [ebx+12]
            pop ebp
            ret
        "#,
            options(noreturn)
        )
    }
}

pub struct CPU;
impl CPU {
    /// # Safety
    /// This function is mother of all unsafety, it switches the page directory to the one mapped by @task
    /// and drops us back into whatever @ip the task has specified in @registers.
    pub unsafe fn return_to_task(task: Shared<Task>) {
        // NOTE: It's important that we copy the registers, since we need to drop
        // the lock on the task for anyone who might need it once the task starts (read syscall)
        let registers = task.with_rlock(|task| {
            Paging::switch(&task.page_directory);
            task.registers
        });

        unsafe { Self::_user_return(&registers) };
    }

    unsafe extern "C" fn _user_return(regs: &Registers) {
        asm!(
            r#"
            mov ebp, esp
            // PUSH THE DATA SEGMENT (SS WILL BE FINE)
            // PUSH THE STACK ADDRESS
            // PUSH THE FLAGS
            // PUSH THE CODE SEGMENT
            // PUSH IP

            // push the data/stack selector
            push ebx
            // Push the stack pointer
            push ecx

            // Push the flags
            // We need to set the IF (Interrupt Enable flag) otherwise the user process might never yield
            pushfd
            pop ecx
            or ecx, 0x200
            push ecx

            // Push the code segment
            push edx

            // Push the IP to execute
            push edi

            // Setup some segment registers
            mov ds, ebx
            mov es, ebx
            mov fs, ebx
            mov gs, ebx

            // Push the @regs
            push eax
            call {}
            add esp, 4

            // Let's leave kernel land and execute in user land!
            iretd

            "#,
            sym Registers::restore,
            in("ebx") regs.ss, in("ecx") regs.sp, in("edx") regs.cs, in("edi") regs.ip, in("eax") regs as *const Registers,
            options(noreturn)
        )
    }
}

#[packed]
pub struct InterruptFrame {
    pub edi: usize,
    pub esi: usize,
    pub ebp: usize,
    pub unused: usize,
    pub ebx: usize,
    pub edx: usize,
    pub ecx: usize,
    pub eax: usize,
    pub errno: usize,
    pub ip: usize,
    pub cs: usize,
    pub flags: usize,
    pub sp: usize,
    pub ss: usize,
}

impl InterruptFrame {
    pub fn errno(&self) -> usize {
        self.errno
    }
}

impl Display for InterruptFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let edi = self.edi;
        let esi = self.esi;
        let ebp = self.ebp;
        let ebx = self.ebx;
        let edx = self.edx;
        let ecx = self.ecx;
        let eax = self.eax;
        let ip = self.ip;
        let cs = self.cs;
        let flags = self.flags;
        let sp = self.sp;
        let ss = self.ss;
        write!(
            f,
            r#"
            edi: 0x{:08x}    esi: 0x{:08x}    ebp: 0x{:08x}
            ebx: 0x{:08x}    edx: 0x{:08x}
            ecx: 0x{:08x}    eax: 0x{:08x}

            flags: 0b{:08b}
            ip: 0x{:08x}     cs: 0x{:08x}
            sp: 0x{:08x}
            ss: 0x{:08x}
        "#,
            edi, esi, ebp, ebx, edx, ecx, eax, flags, ip, cs, sp, ss
        )
    }
}
