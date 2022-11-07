#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

mod uptime;
mod uptime_delay;

extern crate alloc;

use core::{alloc::Layout, fmt::Debug, panic::PanicInfo};

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use embedded_graphics::{
    mono_font::{
        iso_8859_15::{FONT_10X20, FONT_8X13},
        MonoTextStyle,
    },
    pixelcolor::BinaryColor,
    prelude::Point,
    text::{Alignment, Text},
    Drawable,
};
use rp_pico as bsp;

use bsp::{
    entry,
    hal::{
        clocks::ClocksManager,
        pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking, PLLConfig},
        xosc::setup_xosc_blocking,
        Clock, Sio, Watchdog, I2C,
    },
    pac,
};

use alloc_cortex_m::CortexMHeap;

use fugit::{HertzU32, RateExtU32};
use rtt_target::{rprintln, rtt_init_print};
use sh1106::{interface::DisplayInterface, prelude::GraphicsMode, Builder};

use crate::uptime::Uptime;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    init_heap();

    let mut pac = pac::Peripherals::take().unwrap();
    let sio = Sio::new(pac.SIO);

    let core = pac::CorePeripherals::take().unwrap();
    let mut uptime = Uptime::new(core.SYST);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // External crystal on the Pico board is 12 Mhz
    let xtal_freq_hz = 12_000_000u32;
    // let clocks = init_clocks_and_plls(
    //     xtal_freq_hz,
    //     pac.XOSC,
    //     pac.CLOCKS,
    //     pac.PLL_SYS,
    //     pac.PLL_USB,
    //     &mut pac.RESETS,
    //     &mut watchdog,
    // )
    // .ok()
    // .unwrap();

    let xosc = setup_xosc_blocking(pac.XOSC, xtal_freq_hz.Hz())
        .ok()
        .unwrap();

    // Configure watchdog tick generation to tick over every microsecond
    watchdog.enable_tick_generation((xtal_freq_hz / 1_000_000) as u8);

    let mut clocks = ClocksManager::new(pac.CLOCKS);

    // Some SYS_PLL configurations and resulting clock frequencies:
    // 11   MHz      400, 1, 6, 6
    //  8.3 MHz      600, 2, 6, 6
    //  7 083 333 Hz 510, 2, 6, 6

    let pll_sys = setup_pll_blocking(
        pac.PLL_SYS,
        xosc.operating_frequency(),
        PLLConfig {
            vco_freq: HertzU32::MHz(510),
            refdiv: 2,
            post_div1: 6,
            post_div2: 6,
        },
        &mut clocks,
        &mut pac.RESETS,
    )
    .ok()
    .unwrap();
    rprintln!("PLL_SYS freq: {}", pll_sys.operating_frequency());

    let pll_usb = setup_pll_blocking(
        pac.PLL_USB,
        xosc.operating_frequency(),
        PLL_USB_48MHZ,
        &mut clocks,
        &mut pac.RESETS,
    )
    .ok()
    .unwrap();

    clocks.init_default(&xosc, &pll_sys, &pll_usb).unwrap();

    let i2c_disp = I2C::i2c0(
        pac.I2C0,
        pins.gpio8.into_mode(),
        pins.gpio9.into_mode(),
        400.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c_disp).into();

    disp.init().unwrap();
    // Flushing the buffer of a just-initialized display driver clears the display.
    disp.flush().unwrap();

    disp.set_contrast(64).unwrap();

    let mut info_screen = InfoScreen::default();

    loop {
        for _ in 0.. {
            info_screen.draw(&mut disp);
            uptime.delay_ms(900u64);
        }
    }
}

struct InfoScreen {
    msg_index: usize,
    //scroll_x_offset: u32,
}

impl Default for InfoScreen {
    fn default() -> Self {
        Self {
            msg_index: 0,
            // scroll_x_offset: 0,
        }
    }
}

impl InfoScreen {
    fn draw<DI>(&mut self, disp: &mut GraphicsMode<DI>)
    where
        DI: DisplayInterface,
        DI::Error: Debug,
    {
        let first_name = "Raman";
        let last_name = "Fedaseyeu";
        let email = "werediver@pm.me";
        let msgs = ["Signify", "Philips Hue", "Made with Rust", "in a weekend"];

        // let char_style_small = MonoTextStyle::new(&FONT_6X12, BinaryColor::On);
        let char_style_medium = MonoTextStyle::new(&FONT_8X13, BinaryColor::On);
        let char_style_large = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

        disp.clear();

        let mut cursor: Point = Default::default();

        cursor.y += 12;
        cursor = Text::with_alignment(first_name, cursor, char_style_large, Alignment::Left)
            .draw(disp)
            .unwrap();

        cursor.x += 4;
        cursor = Text::with_alignment(last_name, cursor, char_style_medium, Alignment::Left)
            .draw(disp)
            .unwrap();

        cursor.x = cursor.x / 2 - 1;
        cursor.y += char_style_medium.font.baseline as i32 + 8;
        cursor = Text::with_alignment(email, cursor, char_style_medium, Alignment::Center)
            .draw(disp)
            .unwrap();

        // Horizontal scrolling. Too ugly with that low framerate.
        //
        // cursor.x = -(self.scroll_x_offset as i32);
        // cursor.y += char_style_medium.font.baseline as i32 + 16;
        // cursor = Text::with_alignment(company, cursor, char_style_medium, Alignment::Left)
        //     .draw(disp)
        //     .unwrap();
        // if cursor.x <= 0 {
        //     self.scroll_x_offset = 0;
        // }
        // let disp_width = disp.get_dimensions().0 as i32;
        // if cursor.x < disp_width {
        //     _ = Text::with_alignment(company, cursor, char_style_medium, Alignment::Left)
        //         .draw(disp)
        //         .unwrap();
        // } else {
        //     // Render off-screen to keep the timing the same every frame
        //     Text::with_alignment(
        //         company,
        //         Point::new(128, 128),
        //         char_style_medium,
        //         Alignment::Left,
        //     )
        //     .draw(disp)
        //     .unwrap();
        // }
        // self.scroll_x_offset += 6;

        cursor.x = 63;
        cursor.y += char_style_medium.font.baseline as i32 + 16;
        _ = Text::with_alignment(
            msgs[self.msg_index],
            cursor,
            char_style_medium,
            Alignment::Center,
        )
        .draw(disp)
        .unwrap();

        self.msg_index = (self.msg_index + 1) % msgs.len();

        disp.flush().unwrap();
    }
}

#[alloc_error_handler]
fn oom(layout: Layout) -> ! {
    rprintln!(
        "failed to allocate {} bytes aligned on {} bytes)",
        layout.size(),
        layout.align()
    );
    loop {
        cortex_m::asm::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {
        cortex_m::asm::wfi();
    }
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

fn init_heap() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 128 * 1024;
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
}
