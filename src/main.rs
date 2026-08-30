#![no_std]
#![no_main]

mod arch;
mod kernel;

use cortex_m::peripheral::{
    scb::SystemHandler,
    syst::SystClkSource,
};

use cortex_m_rt::{
    entry,
    exception,
};

use panic_halt as _;

use stm32f4xx_hal::{
    pac,
    prelude::*,
    rcc::Config,
};

use kernel::{
    scheduler,
    task::TaskStack,
};

// ============================================================
// Three independent task stacks
// ============================================================

static mut TASK_A_STACK: TaskStack = TaskStack::new();
static mut TASK_B_STACK: TaskStack = TaskStack::new();
static mut TASK_C_STACK: TaskStack = TaskStack::new();

// GPIOA BSRR register.
const GPIOA_BSRR: *mut u32 = 0x4002_0018 as *mut u32;

fn led_on() {
    unsafe {
        core::ptr::write_volatile(
            GPIOA_BSRR,
            1 << 5,
        );
    }
}

fn led_off() {
    unsafe {
        core::ptr::write_volatile(
            GPIOA_BSRR,
            1 << 21,
        );
    }
}

// ============================================================
// TASK A
//
// Forces LED ON.
// ============================================================

fn task_a() -> ! {
    loop {
        led_on();
        cortex_m::asm::nop();
    }
}

// ============================================================
// TASK B
//
// Forces LED OFF.
// ============================================================

fn task_b() -> ! {
    loop {
        led_off();
        cortex_m::asm::nop();
    }
}

// ============================================================
// TASK C
//
// Also forces LED OFF.
//
// This intentionally makes the visual pattern:
//
// ON  250 ms
// OFF 500 ms
//
// because:
//
// A = ON
// B = OFF
// C = OFF
// ============================================================

fn task_c() -> ! {
    loop {
        led_off();
        cortex_m::asm::nop();
    }
}

// ============================================================
// SysTick
// ============================================================

#[exception]
fn SysTick() {
    let ticks = kernel::time::tick();

    // Context switch every 250 ms so it's visually obvious.
    if ticks % 250 == 0 {
        cortex_m::peripheral::SCB::set_pendsv();
    }
}

// ============================================================
// main
// ============================================================

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut cp = cortex_m::Peripherals::take().unwrap();

    // 48 MHz CPU clock.
    let mut rcc = dp.RCC.freeze(
        Config::hsi()
            .sysclk(48.MHz())
    );

    // Configure onboard LD2 / PA5 as output.
    let gpioa = dp.GPIOA.split(&mut rcc);

    let _led = gpioa.pa5.into_push_pull_output();

    // PendSV = lowest priority.
    //
    // SysTick = slightly above PendSV.
    unsafe {
        cp.SCB.set_priority(
            SystemHandler::PendSV,
            0xFF,
        );

        cp.SCB.set_priority(
            SystemHandler::SysTick,
            0xFE,
        );
    }

    // ========================================================
    // Register tasks
    // ========================================================

    unsafe {
        scheduler::spawn(
            core::ptr::addr_of_mut!(TASK_A_STACK),
            task_a,
            2,
        )
        .unwrap();

        scheduler::spawn(
            core::ptr::addr_of_mut!(TASK_B_STACK),
            task_b,
            2,
        )
        .unwrap();

        scheduler::spawn(
            core::ptr::addr_of_mut!(TASK_C_STACK),
            task_c,
            2,
        )
        .unwrap();
    }

    // ========================================================
    // Configure SysTick for 1 kHz.
    //
    // 48 MHz / 1000 = 48,000
    // ========================================================

    cp.SYST.set_clock_source(
        SystClkSource::Core
    );

    cp.SYST.set_reload(
        48_000 - 1
    );

    cp.SYST.clear_current();

    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();

    // Start first registered task.
    scheduler::start();
}