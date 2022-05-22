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

// relative time
pub struct StateData {
    pub a: Duration,
    pub b: Duration,
    pub c: Duration,
}

// 0.1 seconds
const TIMEOUT_CYCLE: Duration = Duration::from_millis(100);


pub struct Workspace<GpioA: gpio::InputPin, GpioB: gpio::InputPin, GpioC: gpio::InputPin> {
    // 3 input pins
    pub recv_a: GpioA,
    pub recv_b: GpioB,
    pub recv_c: GpioC,
}


fn read<GpioA: gpio::InputPin, GpioB: gpio::InputPin, GpioC: gpio::InputPin>(workspace: &Workspace<GpioA, GpioB, GpioC>) -> Result<StateData> {
    panic!("TODO")
}

fn calculate(data: StateData) -> Result<common::ControlData> {
    panic!("TODO")
}

fn send(wifi: &mut EspWifi, data: common::ControlData) -> Result<()> {
    panic!("TODO")
}

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let mut wifi = common::init_wifi()?;

    let workspace = Workspace {
        recv_a: pins.gpio4.into_input()?,
        recv_b: pins.gpio9.into_input()?,
        recv_c: pins.gpio8.into_input()?,
    };

    loop {
        send(&mut *wifi, calculate(read(&workspace)?)?)?;
    }

    Ok(())
}

