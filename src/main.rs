#![no_main]
#![no_std]

use core::panic::PanicInfo;

#[rtic::app(
    device = rp235x_hal::pac,
    peripherals = true,
    dispatchers = [SW0_IRQ]
)]
mod app {
    use fugit::MicrosDurationU32;
    use hal::timer::{Alarm, Alarm0, Timer};
    use rp235x_hal::{self as hal, timer::CopyableTimer0};

    /// External crystal frequency
    const XTAL_FREQ_HZ: u32 = 12_000_000u32;

    // Shared resources between tasks
    #[shared]
    struct Shared {
        // Shared integer resource
        shared_resource: u32,
    }

    // Local resources exclusive to specific tasks
    #[local]
    struct Local {
        // Hardware alarm resource dedicated to the high-priority task
        alarm: Alarm0<CopyableTimer0>,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local) {
        let mut watchdog = hal::Watchdog::new(cx.device.WATCHDOG);
        let clocks = hal::clocks::init_clocks_and_plls(
            XTAL_FREQ_HZ,
            cx.device.XOSC,
            cx.device.CLOCKS,
            cx.device.PLL_SYS,
            cx.device.PLL_USB,
            &mut cx.device.RESETS,
            &mut watchdog,
        )
        .unwrap();
        let mut resets = cx.device.RESETS;
        let mut timer = Timer::new_timer0(cx.device.TIMER0, &mut resets, &clocks);

        let mut alarm = timer.alarm_0().unwrap();
        alarm
            .schedule(MicrosDurationU32::micros(1_000_000))
            .unwrap();
        alarm.enable_interrupt();

        let shared = Shared { shared_resource: 0 };
        let local = Local { alarm };

        (shared, local)
    }

    // High-Priority Task: Triggered directly by the Hardware Timer Interrupt
    // Bound to TIMER0_IRQ_0 (NVIC priority 2)
    #[task(
        binds = TIMER0_IRQ_0,
        priority = 2,
        shared = [shared_resource],
        local = [alarm]
    )]
    fn high_priority_task(mut cx: high_priority_task::Context) {
        cx.local.alarm.clear_interrupt();
        cx.local
            .alarm
            .schedule(MicrosDurationU32::micros(1_000_000))
            .unwrap();

        cx.shared.shared_resource.lock(|resource| {
            *resource += 1;
        });

        // Software-trigger the low priority task via the SPARE_IRQ_0/SW0_IRQ dispatcher
        low_priority_task::spawn().unwrap();
    }

    // Low-Priority Task: Triggered by software spawning
    // Assigned to priority 1 (lower than the timer task)
    #[task(
        priority = 1,
        shared = [shared_resource]
    )]
    async fn low_priority_task(mut cx: low_priority_task::Context) {
        // --- CRITICAL SECTION START ---
        // To read 'shared_resource', the low-priority task must use a lock block.
        // RTIC automatically raises the NVIC priority to 2 during this closure,
        // completely masking out high_priority_task without stopping the sys-clock.
        cx.shared.shared_resource.lock(|resource| {
            let local_copy = *resource;

            let _ = local_copy;
        });
        // --- CRITICAL SECTION END (NVIC drops back to priority 1 automatically) ---
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
