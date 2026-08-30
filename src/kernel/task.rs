use core::ptr;

pub const STACK_WORDS: usize = 256;

pub type TaskId = usize;
pub type TaskEntry = fn() -> !;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Unused,
    Ready,
    Running,
    Sleeping,
    Blocked,
}

#[repr(C, align(8))]
pub struct TaskStack {
    words: [u32; STACK_WORDS],
}

impl TaskStack {
    pub const fn new() -> Self {
        Self {
            words: [0; STACK_WORDS],
        }
    }
}

pub struct TaskControlBlock {
    pub id: TaskId,

    // Saved Process Stack Pointer.
    pub stack_pointer: *mut u32,

    pub state: TaskState,

    // Stored now; scheduler will use this in the next milestone.
    pub priority: u8,

    // Used later by sleep().
    pub wake_tick: u32,
}

impl TaskControlBlock {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            stack_pointer: ptr::null_mut(),
            state: TaskState::Unused,
            priority: 0,
            wake_tick: 0,
        }
    }

    pub unsafe fn new(
        id: TaskId,
        stack: *mut TaskStack,
        entry: TaskEntry,
        priority: u8,
    ) -> Self {
        let stack_pointer = initialize_stack(stack, entry);

        Self {
            id,
            stack_pointer,
            state: TaskState::Ready,
            priority,
            wake_tick: 0,
        }
    }
}

unsafe fn initialize_stack(
    stack: *mut TaskStack,
    entry: TaskEntry,
) -> *mut u32 {
    let base = (*stack).words.as_mut_ptr();

    // Cortex-M stacks grow downward.
    let mut sp = base.add(STACK_WORDS);

    // ========================================================
    // Hardware exception frame
    //
    // Cortex-M automatically restores these when returning
    // from PendSV/SVC.
    // ========================================================

    // xPSR
    // Thumb bit must be set.
    sp = sp.sub(1);
    ptr::write(sp, 0x0100_0000);

    // PC
    // First instruction executed by the task.
    sp = sp.sub(1);
    ptr::write(sp, entry as usize as u32);

    // LR
    sp = sp.sub(1);
    ptr::write(sp, task_exit as usize as u32);

    // R12
    sp = sp.sub(1);
    ptr::write(sp, 0);

    // R3
    sp = sp.sub(1);
    ptr::write(sp, 0);

    // R2
    sp = sp.sub(1);
    ptr::write(sp, 0);

    // R1
    sp = sp.sub(1);
    ptr::write(sp, 0);

    // R0
    sp = sp.sub(1);
    ptr::write(sp, 0);

    // ========================================================
    // Software-saved registers
    //
    // PendSV saves/restores R4-R11.
    // ========================================================

    for _ in 0..8 {
        sp = sp.sub(1);
        ptr::write(sp, 0);
    }

    sp
}

fn task_exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}