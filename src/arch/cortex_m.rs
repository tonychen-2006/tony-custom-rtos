core::arch::global_asm!(
    r#"
    .syntax unified
    .thumb

    // =========================================================
    // SVCall
    //
    // Starts the first RTOS task.
    // =========================================================

    .global SVCall
    .type SVCall, %function
    .thumb_func

SVCall:
    // Ask Rust for Task 0's saved stack pointer.
    //
    // Returned in r0.
    bl rustos_start_sp

    // Restore R4-R11.
    //
    // Double braces are required because this assembly
    // lives inside Rust's global_asm! template.
    ldmia r0!, {{r4-r11}}

    // r0 now points to the hardware exception frame:
    //
    // R0
    // R1
    // R2
    // R3
    // R12
    // LR
    // PC
    // xPSR

    // Install Task 0's stack as the Process Stack Pointer.
    msr psp, r0

    // CONTROL.SPSEL = 1
    //
    // Thread mode now uses PSP instead of MSP.
    movs r0, #2
    msr CONTROL, r0

    isb

    // EXC_RETURN = 0xFFFFFFFD
    //
    // Return to:
    // - Thread mode
    // - using PSP
    // - basic integer exception frame
    ldr lr, =0xFFFFFFFD

    // Cortex-M restores:
    //
    // R0-R3
    // R12
    // LR
    // PC
    // xPSR
    //
    // PC becomes task_a().
    bx lr


    // =========================================================
    // PendSV
    //
    // Performs every task context switch.
    // =========================================================

    .global PendSV
    .type PendSV, %function
    .thumb_func

PendSV:
    // When we arrive here, hardware has ALREADY pushed:
    //
    // R0-R3
    // R12
    // LR
    // PC
    // xPSR
    //
    // onto the current task's PSP.

    // Get the current Process Stack Pointer.
    mrs r0, psp

    // Save R4-R11 ourselves.
    stmdb r0!, {{r4-r11}}

    // Preserve PendSV's EXC_RETURN value.
    //
    // Two registers keep the MSP 8-byte aligned before
    // calling a Rust function.
    push {{r3, lr}}

    // r0 contains outgoing task SP.
    //
    // rustos_switch_context:
    //   - stores outgoing SP
    //   - chooses next task
    //   - returns incoming SP in r0
    bl rustos_switch_context

    // Recover EXC_RETURN.
    pop {{r3, lr}}

    // Restore incoming task's R4-R11.
    ldmia r0!, {{r4-r11}}

    // PSP now points to the incoming hardware exception frame.
    msr psp, r0

    // Exception return.
    //
    // Hardware restores the remainder of the registers and
    // continues from the incoming task's saved PC.
    bx lr
"#
);