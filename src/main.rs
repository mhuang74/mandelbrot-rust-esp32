#![allow(clippy::single_component_path_imports)]
// #![feature(backtrace)]

#[cfg(all(feature = "qemu", not(esp32)))]
compile_error!("The `qemu` feature can only be built for the `xtensa-esp32-espidf` target.");

use std::sync::{Condvar, Mutex};
use std::{sync::Arc, thread, time::*};

use anyhow::*;
use log::*;


//use embedded_svc::anyerror::*;
//use embedded_svc::eth;
//use embedded_svc::eth::Eth;
use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
//use embedded_svc::ipv4;
use embedded_svc::wifi::*;

//use esp_idf_svc::eth::*;
use esp_idf_svc::httpd as idf;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::sysloop::*;
#[allow(unused_imports)]
use esp_idf_svc::wifi::*;

use esp_idf_sys;

#[allow(dead_code)]
#[cfg(not(feature = "qemu"))]
const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
#[allow(dead_code)]
#[cfg(not(feature = "qemu"))]
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

//const MAX_BOUNDS: (usize, usize) = (64, 64);

// statically allocate image buffers
// static mut PIXELS:[u8;  MAX_BOUNDS.0 * MAX_BOUNDS.1] = [0; MAX_BOUNDS.0 * MAX_BOUNDS.1];
// statically allocate buffer for encoded image; assume jpeg gives at least 5:1 compression
// static mut ENCODED:[u8; (MAX_BOUNDS.0 / 5 as usize) * (MAX_BOUNDS.1 / 5 as usize)] = [0; (MAX_BOUNDS.0 / 5 as usize) * (MAX_BOUNDS.1 / 5 as usize)];

fn print_heap_info() {
    unsafe {
        // esp_idf_sys::heap_caps_print_heap_info(esp_idf_sys::MALLOC_CAP_8BIT);
        
        let min_free_8bit_cap = esp_idf_sys::heap_caps_get_minimum_free_size(esp_idf_sys::MALLOC_CAP_8BIT);
        info!("Min Free DRAM:\t{}", min_free_8bit_cap);

        // esp_idf_sys::heap_caps_check_integrity_all(true);

    }


}

fn main() -> Result<()> {
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello from Mandelbrot-ESP!");
    print_heap_info();

    // Get backtraces from anyhow; only works for Xtensa arch currently
    // #[cfg(arch = "xtensa")]
    // env::set_var("RUST_BACKTRACE", "1");

    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    info!("before network start");
  
    #[cfg(not(feature = "qemu"))]
    #[allow(unused_mut)]
    let mut wifi = wifi(
        netif_stack.clone(),
        sys_loop_stack.clone(),
        default_nvs.clone(),
    )?;

    #[cfg(feature = "qemu")]
    let eth = eth_configure(Box::new(EspEth::new_openeth(
        netif_stack.clone(),
        sys_loop_stack.clone(),
    )?))?;    

    let mutex: Arc<(std::sync::Mutex<Option<u32>>, Condvar)> = Arc::new((Mutex::new(None), Condvar::new()));

    info!("before httpd start");

    let httpd = httpd()?;

    let mut wait = mutex.0.lock().unwrap();

    #[allow(unused)]
    let cycles = loop {
        if let Some(cycles) = *wait {
            break cycles;
        } else {
            wait = mutex.1.wait(wait).unwrap();
        }
    };

    for s in 0..3 {
        info!("Shutting down in {} secs", 3 - s);
        thread::sleep(Duration::from_secs(1));
    }

    drop(httpd);
    info!("Httpd stopped");

    #[cfg(not(feature = "qemu"))]
    {
        drop(wifi);
        info!("Wifi stopped");
    }

    #[cfg(any(feature = "qemu"))]
    {
        let _eth_peripherals = eth.release()?;
        info!("Eth stopped");
    } 

    Ok(())
}

use num::Complex;
mod mandelbrot;
use image::{ColorType, codecs::bmp::BmpEncoder};


fn handle_mandelbrot(_req: Request) -> Result<Response, Error> {
    info!("Handling Mandelbrot request");

    // Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20
    let bounds = (100, 85);
    let upper_left = Complex { re: -1.20, im: 0.35};
    let lower_right = Complex { re: -1.0, im: 0.20};

    let mut PIXELS = vec![0; bounds.0 * bounds.1];
    let mut ENCODED = Vec::new();

    unsafe {
        mandelbrot::render(&mut PIXELS, bounds, upper_left, lower_right);
        info!("Mandelbrot rendered!");
        print_heap_info();

        BmpEncoder::new(&mut ENCODED)
            .encode(&PIXELS, bounds.0 as u32, bounds.1 as u32, ColorType::L8)
            .expect("Unable to encode image");

        info!("Mandelbrot converted to jpeg!");

        let response = Response::new(200)
            .content_type("image/bmp")
            .content_len(ENCODED.len())
            .header("Content-Disposition", "inline; filename=mandel.bmp")
            .header("Access-Control-Allow-Origin", "*")
            // .header("X-Timestamp", SystemTime::now())
            .body(Body::from(ENCODED))
            ;
        info!("Created Mandelbrot response");

        Ok(response)
    }


}

#[allow(unused_variables)]
fn httpd() -> Result<idf::Server> {
    let server = idf::ServerRegistry::new()
        .at("/")
        .get(|_| Ok("Hello, world!".into()))?
        .at("/mandelbrot")
        .get(handle_mandelbrot)?
        .at("/stop")
        .get(|_| bail!("Stopping server!"))?
        .at("/secret")
        .get(|_| {
            Response::new(403)
                .status_message("No permissions")
                .body("You have no permissions to access this page".into())
                .into()
        })?;


    server.start(&Default::default())
}


#[cfg(not(feature = "qemu"))]
#[allow(dead_code)]
fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to connect to hidden SSID");

    let channel = None; // using hidden SSID, so channel is unknown

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(_ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

#[cfg(any(feature = "qemu"))]
fn eth_configure<HW>(mut eth: Box<EspEth<HW>>) -> Result<Box<EspEth<HW>>> {
    info!("Eth created");

    eth.set_configuration(&eth::Configuration::Client(Default::default()))?;

    info!("Eth configuration set, about to get status");

    let status = eth.get_status();

    if let eth::Status::Started(eth::ConnectionStatus::Connected(eth::IpStatus::Done(Some(
        _ip_settings,
    )))) = status
    {
        info!("Eth connected");
    } else {
        bail!("Unexpected Eth status: {:?}", status);
    }

    Ok(eth)
}





