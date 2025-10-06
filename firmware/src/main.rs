#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m;
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::{
    pac::{self, DWT},
    prelude::*,
    serial::{config, Serial},
};

// ----------------------- Protocol Vars ----------------------------
const START_BYTE: u8 = 0xFF;
const END_BYTE: u8 = 0xFE;
const BLINK_RATE_BYTE: u8 = 0x1;
const REBOOT_BYTE: u8 = 0x2;

// I wanted some verbose output on panics, so this is basicallt panic_halt crate, but with a print
// message slid into it
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    rprintln!(
        "MESSAGE: {0} | AT {1}",
        info.message(),
        info.location().unwrap()
    );
    loop {}
}

#[entry]
fn main() -> ! {
    rtt_init_print!(); // RTT for debuging and status

    // -----------------------  Setup  ----------------------------
    // Because I am not using and delay() operation, as that would break the timing
    // for transmissions, I keep track of the clock to establish a delay for the LED blink
    let mut cor_per = cortex_m::Peripherals::take().unwrap();
    cor_per.DCB.enable_trace();
    cor_per.DWT.enable_cycle_counter();

    let peripherals = pac::Peripherals::take().unwrap();
    let mut rcc = peripherals.RCC.constrain();
    let mut flash = peripherals.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(72.MHz())
        .freeze(&mut flash.acr);

    let mut gpio_a = peripherals.GPIOA.split(&mut rcc.ahb);
    let mut gpio_c = peripherals.GPIOC.split(&mut rcc.ahb);

    // LED and Button pins
    let mut led = gpio_a // D13
        .pa5
        .into_push_pull_output(&mut gpio_a.moder, &mut gpio_a.otyper);
    let _btn = gpio_c // USR_BTN
        .pc13
        .into_pull_down_input(&mut gpio_c.moder, &mut gpio_c.pupdr);

    // UART pins and init
    let uart_tx_pin =
        gpio_a
            .pa2
            .into_af_push_pull(&mut gpio_a.moder, &mut gpio_a.otyper, &mut gpio_a.afrl);
    let uart_rx_pin =
        gpio_a
            .pa3
            .into_af_push_pull(&mut gpio_a.moder, &mut gpio_a.otyper, &mut gpio_a.afrl);

    let mut uart = Serial::new(
        peripherals.USART2,
        (uart_tx_pin, uart_rx_pin),
        config::Config::default().baudrate(9600.Bd()),
        clocks,
        &mut rcc.apb1,
    );

    // -----------------------  Config/State  ----------------------------
    // UART
    let mut packet_buffer = [0u8; 2]; // This stored every packet. Don't keep the START or the END byte, because there is no need to keep them.
    let mut index = 0; // This keeps track of which byte of the packet I am supposed to read now
    let mut reading_packet = false; // This indicates if I have received a START byte

    // LED
    const REFERENCE_BLINK: u32 = 1_000_000; // This is kinda arbitrary, just felt right. At 72MHz, this would be 1/72 of a second
    let mut last_toggle = DWT::cycle_count();
    let mut led_interval_cycles: u32 = REFERENCE_BLINK * 100; // The default blink rate is the middle of the renge

    loop {
        let byte = match uart.read() {
            Ok(byte) => {
                rprintln!("Reading byte: {:#02x}", byte);
                byte
            }
            // The Serial implementation in stm32f3xx_hal uses the nb crate
            // This then returns a nb::Result, which means it has WouldBlock ErrorKind avalible
            // Every cycle when .read() doesn't have anything in the buffer, it returns Err(nb::Error::WouldBlock)
            Err(_) => {
                // Can take advantage of it by executing the non-blocking code like led blink and button press here

                let now = DWT::cycle_count();
                if now.wrapping_sub(last_toggle) >= led_interval_cycles {
                    led.toggle().unwrap();
                    last_toggle = now;
                }

                continue;
            }
        };

        // Handling the START byte
        if !reading_packet && byte == START_BYTE {
            reading_packet = true;
            index = 0;
            continue;
        }

        // Handling the END byte and COMMANDs
        if reading_packet && byte == END_BYTE {
            reading_packet = false;
            index = 0;

            match packet_buffer[0] {
                BLINK_RATE_BYTE => {
                    // BLINK_RATE command
                    led_interval_cycles = packet_buffer[1] as u32 * REFERENCE_BLINK;
                }
                REBOOT_BYTE => {
                    // REBOOT command. I think the main use case is to reset the blink rate?
                    // https://users.rust-lang.org/t/programatically-resetting-an-embedded-system-arm-cortex/82226/3
                    cortex_m::peripheral::SCB::sys_reset();
                }
                _ => { /* Ignore any other commands */ }
            }

            continue;
        }

        packet_buffer[index] = byte;
        index += 1;
    }
}
