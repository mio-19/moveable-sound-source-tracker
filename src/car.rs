#![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]

mod common;

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};
use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};

use anyhow::bail;

use embedded_svc::mqtt::client::utils::ConnState;
use log::*;

use url;

use smol;

use embedded_hal::adc::OneShot;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;

use embedded_svc::eth;
use embedded_svc::eth::{Eth, TransitionalState};
use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
use embedded_svc::io;
use embedded_svc::ipv4;
use embedded_svc::mqtt::client::{Client, Connection, MessageImpl, Publish, QoS};
use embedded_svc::ping::Ping;
use embedded_svc::sys_time::SystemTime;
use embedded_svc::timer::TimerService;
use embedded_svc::timer::*;
use embedded_svc::wifi::*;

use esp_idf_svc::eth::*;
use esp_idf_svc::eventloop::*;
use esp_idf_svc::eventloop::*;
use esp_idf_svc::httpd as idf;
use esp_idf_svc::httpd::ServerRegistry;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sntp;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::timer::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::adc;
use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::i2c;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use esp_idf_sys::{self, c_types};
use esp_idf_sys::{esp, EspError};

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;

use ili9341;
use ssd1306;
use ssd1306::mode::DisplayConfig;
use st7789;

use epd_waveshare::{epd4in2::*, graphics::VarDisplay, prelude::*};

use esp_idf_hal::ledc::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use std::{borrow::Borrow, time::Duration};
use esp_idf_hal::ledc;


// reference https://github.com/esp-rs/esp-idf-hal/blob/447fcc3616e3a3643ca109d4bc7acf40754da9af/examples/ledc-threads.rs

struct EnginePWMChannel<C0, H0, T0, P0, C1, H1, T1, P1> where
    C0: HwChannel,
    H0: HwTimer,
    T0: Borrow<ledc::Timer<H0>>,
    P0: gpio::OutputPin,
    C1: HwChannel,
    H1: HwTimer,
    T1: Borrow<ledc::Timer<H1>>,
    P1: gpio::OutputPin,
{
    positive: Channel<C0, H0, T0, P0>,
    negative: Channel<C1, H1, T1, P1>,
}

struct CarHardware<C0, H0, T0, P0, C1, H1, T1, P1, C2, H2, T2, P2, C3, H3, T3, P3> where
    C0: HwChannel,
    H0: HwTimer,
    T0: Borrow<ledc::Timer<H0>>,
    P0: gpio::OutputPin,
    C1: HwChannel,
    H1: HwTimer,
    T1: Borrow<ledc::Timer<H1>>,
    P1: gpio::OutputPin,
    C2: HwChannel,
    H2: HwTimer,
    T2: Borrow<ledc::Timer<H2>>,
    P2: gpio::OutputPin,
    C3: HwChannel,
    H3: HwTimer,
    T3: Borrow<ledc::Timer<H3>>,
    P3: gpio::OutputPin,
{
    engine1: EnginePWMChannel<C0, H0, T0, P0, C1, H1, T1, P1>,
    engine2: EnginePWMChannel<C2, H2, T2, P2, C3, H3, T3, P3>,
}

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();

    let mut wifi = common::init_wifi()?;


    let config = config::TimerConfig::default().frequency(25.kHz().into());
    let timer = Arc::new(ledc::Timer::new(peripherals.ledc.timer0, &config)?);
    let channel0 = Channel::new(peripherals.ledc.channel0, timer.clone(), peripherals.pins.gpio4)?;
    let channel1 = Channel::new(peripherals.ledc.channel1, timer.clone(), peripherals.pins.gpio5)?;
    let channel2 = Channel::new(peripherals.ledc.channel2, timer.clone(), peripherals.pins.gpio6)?;
    let channel3 = Channel::new(peripherals.ledc.channel3, timer.clone(), peripherals.pins.gpio7)?;

    loop {}

    Ok(())
}

