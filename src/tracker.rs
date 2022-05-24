#![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]

mod common;

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};
use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};
use std::thread::JoinHandle;

use anyhow::bail;
use arc_swap::ArcSwap;

use embedded_svc::mqtt::client::utils::ConnState;
use log::*;

use url;

use smol;

use embedded_hal::adc::OneShot;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{InputPin, OutputPin, PinState};

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
    pub a: Instant,
    pub b: Instant,
    pub c: Instant,
}

// 0.1 seconds
const VALID_TIME: Duration = Duration::from_millis(100);
// 0.05 seconds
const SOUND_RANGE_TIME: Duration = Duration::from_millis(50);


pub struct Workspace<GpioA: gpio::InputPin, GpioB: gpio::InputPin, GpioC: gpio::InputPin> {
    // 3 input pins
    pub recv_a: GpioA,
    pub recv_b: GpioB,
    pub recv_c: GpioC,
}

const RECV_VALID: bool = true;
const RECV_INVALID: bool = !RECV_VALID;

fn read_loop<CB: FnMut(StateData) -> Result<()>, GpioA: gpio::InputPin + InputPin, GpioB: gpio::InputPin + InputPin, GpioC: gpio::InputPin + InputPin>(workspace: &Workspace<GpioA, GpioB, GpioC>, mut callback: CB) -> Result<()>
    where
        <GpioA as embedded_hal::digital::v2::InputPin>::Error: std::error::Error + Sync + Send + 'static,
        <GpioB as embedded_hal::digital::v2::InputPin>::Error: std::error::Error + Sync + Send + 'static,
        <GpioC as embedded_hal::digital::v2::InputPin>::Error: std::error::Error + Sync + Send + 'static,
{
    let now = Instant::now();
    let mut last_a = now;
    let mut last_b = now;
    let mut last_c = now;
    let mut last_a_val = RECV_INVALID;
    let mut last_b_val = RECV_INVALID;
    let mut last_c_val = RECV_INVALID;
    loop {
        let a_val = workspace.recv_a.is_high()? == RECV_VALID;
        let b_val = workspace.recv_b.is_high()? == RECV_VALID;
        let c_val = workspace.recv_c.is_high()? == RECV_VALID;
        let now = Instant::now();

        if a_val != last_a_val {
            last_a_val = a_val;
            last_a = now;
        }
        if b_val != last_b_val {
            last_b_val = b_val;
            last_b = now;
        }
        if c_val != last_c_val {
            last_c_val = c_val;
            last_c = now;
        }

        let last_earlistest = if last_a <= last_b && last_a <= last_c {
            last_a
        } else if last_b <= last_a && last_b <= last_c {
            last_b
        } else {
            if !(last_c <= last_a && last_c <= last_b) {
                panic!("last_c is not earlistest");
            }
            last_c
        };

        if a_val &&
            b_val &&
            c_val &&
            now.duration_since(last_a) > VALID_TIME &&
            now.duration_since(last_b) > VALID_TIME &&
            now.duration_since(last_c) > VALID_TIME &&
            last_a.duration_since(last_earlistest) < SOUND_RANGE_TIME &&
            last_b.duration_since(last_earlistest) < SOUND_RANGE_TIME &&
            last_c.duration_since(last_earlistest) < SOUND_RANGE_TIME {
            let data = StateData {
                a: last_a,
                b: last_b,
                c: last_c,
            };
            callback(data)?;
        }
    }
    Ok(())
}


fn calculate(data: StateData) -> Result<common::ControlData> {
    if data.a >= data.b {
        Ok(common::ControlData { offset: data.a.duration_since(data.b).as_nanos() as i128 })
    } else {
        Ok(common::ControlData { offset: -(data.b.duration_since(data.a).as_nanos() as i128) })
    }
}

fn send_server(data: Arc<ArcSwap<common::ControlData>>) -> Result<()> {
    fn bind_accept(data: Arc<ArcSwap<common::ControlData>>) -> Result<()> {
        info!("About to bind the service to port 8080");

        let listener = TcpListener::bind("0.0.0.0:8080")?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!("Accepted client");

                    let data = data.clone();
                    thread::spawn(move || {
                        handle_client(data, stream);
                    });
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }

        unreachable!()
    }

    fn handle_client(data: Arc<ArcSwap<common::ControlData>>, mut stream: TcpStream) {
        loop {
            stream.write_all(data.load().to_slice()).unwrap();
        }
    }

    thread::spawn(|| bind_accept(data).unwrap());

    Ok(())
}


fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let mut wifi = common::init_wifi_server()?;

    let workspace = Workspace {
        recv_a: pins.gpio4.into_input()?,
        recv_b: pins.gpio9.into_input()?,
        recv_c: pins.gpio8.into_input()?,
    };

    let control = Arc::new(ArcSwap::from(Arc::new(common::ControlData::empty())));

    send_server(control.clone())?;

    read_loop(&workspace, |data| {
        control.store(Arc::new(calculate(data)?));
        Ok(())
    })?;

    Ok(())
}

