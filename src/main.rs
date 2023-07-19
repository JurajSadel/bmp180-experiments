#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embedded_hal::watchdog::WatchdogDisable;

use esp_backtrace as _;
use esp_println::println;
use hal::clock::ClockControl;
// use hal::clock::CpuClock;
use hal::embassy;
use hal::i2c::I2C;
use hal::peripherals::{Peripherals, I2C0};
use hal::prelude::_fugit_RateExtU32;
use hal::prelude::*;
use hal::timer::TimerGroup;
use hal::Rtc;
use hal::IO;

use embassy_executor::Executor;
use embassy_time::{Duration, Timer};

use static_cell::StaticCell;

use crate::bmp180::Bmp180;
// #[cfg(feature = "async")]
// use crate::bmp180::asynch;
mod bmp180;

const INTERVALL_MS: u64 = 1 * 60 * 1000; // 1 minute intervall

#[embassy_executor::task]
async fn measure(i2c: I2C<'static, I2C0>) {
    println!("Hello?"); //printed
    let mut bmp = Bmp180::new(i2c).await;
    println!("Hello?"); //printed

    loop {
        bmp.measure().await; //looks wrong, move error

        Timer::after(Duration::from_millis(1000)).await;
        println!("Hello?"); //printed
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    init_logger();

    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the watchdog timers. For the ESP32-C3, this includes the Super WDT,
    // the RTC WDT, and the TIMG WDTs.
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    println!("preinit");

    #[cfg(feature = "embassy-time-systick")]
    embassy::init(
        &clocks,
        hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );
    println!("postinit");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Create a new peripheral object with the described wiring
    // and standard I2C clock speed
    let i2c0 = I2C::new(
        peripherals.I2C0,
        io.pins.gpio10,
        io.pins.gpio8,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(measure(i2c0)).ok();
    });

    // loop {println!("Measuring");}
    
}

pub fn init_logger() {
    unsafe {
        log::set_logger_racy(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Info);
    }
}

static LOGGER: SimpleLogger = SimpleLogger;
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
