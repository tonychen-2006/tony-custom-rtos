use cortex_m::peripheral::SCB;

use super::task::{
    TaskControlBlock,
    TaskEntry,
    TaskId,
    TaskStack,
    TaskState,
};

pub const MAX_TASKS: usize = 8;

#[derive(Debug)]
pub enum SpawnError {
    TaskTableFull,
}

static mut TASKS: [TaskControlBlock; MAX_TASKS] = [
    TaskControlBlock::empty(),
    TaskControlBlock::empty(),
    TaskControlBlock::empty(),
    TaskControlBlock::empty(),
    TaskControlBlock::empty(),
    TaskControlBlock::empty(),
    TaskControlBlock::empty(),
    TaskControlBlock::empty(),
];

// Number of tasks actually registered.
static mut TASK_COUNT: usize = 0;

// Task currently running.
static mut CURRENT_TASK: usize = 0;

/// Register a new task with the kernel.
///
/// For now tasks are created before scheduler::start().
///
/// Later we can turn this into a safer public TaskBuilder API.
pub unsafe fn spawn(
    stack: *mut TaskStack,
    entry: TaskEntry,
    priority: u8,
) -> Result<TaskId, SpawnError> {
    if TASK_COUNT >= MAX_TASKS {
        return Err(SpawnError::TaskTableFull);
    }

    let id = TASK_COUNT;

    let task = TaskControlBlock::new(
        id,
        stack,
        entry,
        priority,
    );

    TASKS[id] = task;

    TASK_COUNT += 1;

    Ok(id)
}

/// Number of registered tasks.
pub fn task_count() -> usize {
    unsafe { TASK_COUNT }
}

/// Select the next Ready task.
///
/// V0 implementation:
/// round-robin across registered tasks.
///
/// Later this becomes:
/// highest-priority Ready task + round-robin among equal priorities.
unsafe fn choose_next_task() -> usize {

    if TASK_COUNT == 0 {
        return 0;
    }

    let mut highest_priority: Option<u8> = None;

    for i in 0..TASK_COUNT {

        if TASKS[i].state == TaskState::Ready {
            match highest_priority {
                Some(priority) => {
                    if TASKS[i].priority > priority {
                        highest_priority = Some(TASKS[i].priority);
                    }
                }

                None => {
                    highest_priority = Some(TASKS[i].priority);
                }
            }
        }
    }

    let highest_priority = match highest_priority {
        Some(priority) => priority,
        None => return CURRENT_TASK;
    };

    for offset in 1..=TASK_COUNT {
        let candidate = (CURRENT_TASK + offset) % TASK_COUNT;

        if TASKS[candidate].state == TaskState::Ready && TASKS[candidate].priority == highest_priority {
            return candidate;
        }
    }

    CURRENT_TASK
}

/// Called by the SVC assembly handler.
///
/// Gives SVC the first task's initial stack pointer.
#[no_mangle]
pub unsafe extern "C" fn rustos_start_sp() -> *mut u32 {
    if TASK_COUNT == 0 {
        loop {
            cortex_m::asm::bkpt();
        }
    }

    CURRENT_TASK = 0;
    TASKS[CURRENT_TASK].state = TaskState::Running;

    TASKS[CURRENT_TASK].stack_pointer
}

/// Called by PendSV.
///
/// Saves the old task's PSP and returns the PSP belonging
/// to the task that should run next.
#[no_mangle]
pub unsafe extern "C" fn rustos_switch_context(
    current_sp: *mut u32,
) -> *mut u32 {
    // Save outgoing task.
    TASKS[CURRENT_TASK].stack_pointer = current_sp;

    TASKS[CURRENT_TASK].state = TaskState::Ready;

    // Pick the next runnable task.
    let next = choose_next_task();

    CURRENT_TASK = next;

    TASKS[CURRENT_TASK].state = TaskState::Running;

    // Return its saved stack pointer to PendSV.
    TASKS[CURRENT_TASK].stack_pointer
}

/// Voluntarily ask the kernel to reschedule.
///
/// SysTick currently performs automatic preemption,
/// so normal tasks don't need to call this.
pub fn yield_now() {
    SCB::set_pendsv();
}

/// Begin task execution.
///
/// SVC performs the first transition from MSP/main()
/// to Task 0 running with PSP.
pub fn start() -> ! {
    unsafe {
        core::arch::asm!("svc 0");
    }

    loop {
        cortex_m::asm::wfi();
    }
}

pub fn current_task_id() -> TaskId {
    unsafe { CURRENT_TASK }
}

pub fn current_priority() -> u8 {
    unsafe {
        TASKS[CURRENT_TASK].priority
    }
}