use crate::trapframe::TrapFrame;
use crate::user_trap::usertrapret;
use crate::vm::{new_user_page_table, KERNEL_PAGE_TABLE};
use alloc::boxed::Box;
use alloc::string::String;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::usize;
use page_alloc::{PAGE_ALLOCATOR, PAGE_SIZE};
use page_table::entry::addr::VirtualAddr;
use page_table::entry::perm::PTEPermission;
use page_table::PageTable;

core::arch::global_asm!(include_str!("asm/trampoline.S"));

extern "C" {
    static _trampoline: u8;
}

pub enum ProcState {
    Unused,
    Used,
    Sleeping,
    Runnable,
    Running,
    Zombie,
}

#[repr(C)]
pub struct ProcContext {
    // Registers
    pub ra: u64,
    pub sp: u64,
    pub s: [u64; 12],
}

pub(crate) struct PageTableAddr(pub NonNull<PageTable>);
unsafe impl Send for PageTableAddr {}

pub(crate) struct Proc {
    // TODO : Add other things
    pub state: ProcState,
    pub context: ProcContext,

    pub name: String,
    pub pid: usize,

    pub kernel_stack: VirtualAddr,
    // pub memory_size: u64,
    pub page_table: Box<PageTable>,
    pub trap_frame: Box<TrapFrame>,
}
unsafe impl Send for Proc {}

impl Proc {
    pub fn init_user_proc(code: &[u8]) -> Self {
        let kstack = usize::from(PAGE_ALLOCATOR.kalloc().unwrap().addr()) as u64;
        // let trap_frame = NonNull::new(unsafe { &mut *(PAGE_ALLOCATOR.kalloc().unwrap().cast().as_ptr()) }).unwrap();
        let trap_frame = Box::new(TrapFrame::new());
        let mut proc = Self {
            state: ProcState::Unused,
            context: ProcContext {
                ra: usertrapret as usize as u64,
                sp: kstack + PAGE_SIZE as u64,
                s: [0; 12],
            },
            name: String::from("Test Proc Name"),
            pid: get_new_pid(),
            kernel_stack: VirtualAddr::new(kstack),
            // memory_size: 0,
            page_table: new_user_page_table(unsafe { trap_frame.as_ref() }),
            trap_frame,
        };

        // Put the code
        assert!(code.len() <= PAGE_SIZE);
        let ptr = PAGE_ALLOCATOR.kalloc().unwrap().as_ptr();
        for (i, &c) in code.iter().enumerate() {
            unsafe {
                ptr.byte_add(i).write(c);
            }
        }
        let ptr_code = VirtualAddr::new(ptr as usize as u64);
        let (pa, _) = KERNEL_PAGE_TABLE.lock().get_phys_addr_perm(&ptr_code);
        proc.page_table.as_mut().map_pages(
            VirtualAddr::new(0),
            pa,
            PAGE_SIZE,
            PTEPermission::read() | PTEPermission::execute() | PTEPermission::user(),
            0,
        );

        proc
    }
}

fn get_new_pid() -> usize {
    static PID_COUNT: AtomicUsize = AtomicUsize::new(0);
    PID_COUNT.fetch_add(1, Ordering::AcqRel)
}
