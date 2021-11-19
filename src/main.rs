#![allow(clippy::single_component_path_imports)]
#![feature(backtrace)]

#[cfg(all(feature = "qemu", not(esp32)))]
compile_error!("The `qemu` feature can only be built for the `xtensa-esp32-espidf` target.");

use std::sync::{Condvar, Mutex};
use std::usize;
use std::{sync::Arc, thread, time::*};
use std::collections::{HashMap};
use std::io::{Write};

use anyhow::*;
use log::*;

#[allow(unused_imports)]
use embedded_svc::anyerror::*;
#[allow(unused_imports)]
use embedded_svc::eth;
#[allow(unused_imports)]
use embedded_svc::eth::Eth;
use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
#[allow(unused_imports)]
use embedded_svc::ipv4;
#[allow(unused_imports)]
use embedded_svc::wifi::*;

#[allow(unused_imports)]
use esp_idf_svc::eth::*;
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


fn print_heap_info() {
    unsafe {
        esp_idf_sys::heap_caps_print_heap_info(esp_idf_sys::MALLOC_CAP_8BIT);
    }
}


fn main() -> Result<()> {
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Get backtraces from anyhow; only works for Xtensa arch currently
    #[cfg(arch = "xtensa")]
    env::set_var("RUST_BACKTRACE", "1");

    info!("Hello from Mandelbrot-ESP!");

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

    info!("before httpd start");
    let httpd = httpd()?;
    info!("after httpd start");

    let mutex: Arc<(std::sync::Mutex<Option<u32>>, Condvar)> = Arc::new((Mutex::new(None), Condvar::new()));
    let mut wait = mutex.0.lock().unwrap();
    #[allow(unused)]
    let cycles = loop {
        if let Some(cycles) = *wait {
            break cycles;
        } else {
            wait = mutex.1.wait(wait).unwrap();
        }
    };

    for s in (0..3).rev() {
        info!("Shutting down in {} secs", s);
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


    info!("That's all, folks!");
    Ok(())
}

use num::Complex;
mod mandelbrot;
mod encoder;

fn handle_mandelbrot(_req: Request) -> Result<Response, Error> {
    let query_string = _req.query_string().unwrap_or_default();
    let query_params = querystring::querify(&query_string);

    info!("Handling Mandelbrot request. Params:{:?}", query_params);

    let mut param_hash: HashMap<&str,&str> = HashMap::new();
    for (k, v) in &query_params {
        param_hash.insert(k,v);
    }

    let width: usize = param_hash["width"].parse().unwrap_or(640 as usize);
    let height: usize = param_hash["height"].parse().unwrap_or(480 as usize);
    let bounds: (usize, usize) = (width, height);

    let pixel_buffer_size: usize = bounds.0 * bounds.1;
    // assume 1024 bytes enough for image header and other metadata
    let image_buffer_size: usize = pixel_buffer_size + 1024 as usize;

    // Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20
    const upper_left: Complex<f32> = Complex { re: -1.20, im: 0.35};
    const lower_right: Complex<f32> = Complex { re: -1.0, im: 0.20};

    info!("Rendering ({},{}) image from {:?} to {:?}", bounds.0, bounds.1, upper_left, lower_right);

    // use Vec (heap) as pixel buffer to avoid stack overflow
    let mut pixel_buffer: Vec<u8> = Vec::with_capacity(pixel_buffer_size);

    mandelbrot::render(&mut pixel_buffer, bounds, upper_left, lower_right)?;
    info!("Mandelbrot image rendered!");
    
    // use Vector (heap) as image buffer; add extra room for image metadata
    let mut encode_buffer: Vec<u8> = Vec::with_capacity(image_buffer_size);

    // turn Vector into Writable via Cursor
    let mut writable_buffer = std::io::Cursor::new(&mut encode_buffer);

    // use patched BMP encoder from image crate
    encoder::encode_grayscale(&mut writable_buffer, &pixel_buffer, bounds.0 as u32, bounds.1 as u32)?;
    info!("Mandelbrot image encoded!");

    let body_len = writable_buffer.get_ref().len();
    let body = Body::Bytes(encode_buffer);

    let response = Response::new(200)
        .content_type("image/bmp")
        .content_len(body_len)
        .header("Content-Disposition", "inline; filename=mandel.bmp")
        .header("Access-Control-Allow-Origin", "*")
        .body(body)
        ;
    info!("Created Mandelbrot image response");

    Ok(response)
    
}

use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum WriteMethod {
    Direct,
    Write,
    WriteU8,
}

impl FromStr for WriteMethod {
    type Err = ();

    fn from_str(input: &str) -> Result<WriteMethod, Self::Err> {
        match &(input.to_ascii_lowercase())[..] {
            "direct"    => Ok(WriteMethod::Direct),
            "write"     => Ok(WriteMethod::Write),
            "writeu8"   => Ok(WriteMethod::WriteU8),
            _           => Err(()),
        }
    }
}

use byteorder::{WriteBytesExt};

fn test_memory_allocation(kb_blocks:usize, step:usize, method:WriteMethod) -> Result<Vec<u8>, Error> {
    const KILOBYTE: usize = 1024;

    let mut my_vec: Vec<u8> = vec![];

    for i in (step..=kb_blocks).step_by(step) {
        let size = i * KILOBYTE;
        my_vec = Vec::with_capacity(size);
        info!("{}: allocated Vec<u8> of size: {}", i, size);

        match method {
            WriteMethod::Direct => {

                for j in 0..size {
                    my_vec.push(j as u8);
                }
            },
            WriteMethod::Write => {

                // turn Vector into Writable via Cursor
                let mut cursored  = std::io::Cursor::new(&mut my_vec);
                let mut writable_buffer: &mut Write = &mut cursored;
                
                for val in 0u8..=255u8 {
                    writable_buffer.write_all(&[val, val, val, 0] )?;
                }

            },
            WriteMethod::WriteU8 => {

                // turn Vector into Writable via Cursor
                let mut cursored  = std::io::Cursor::new(&mut my_vec);
                let mut writable_buffer: &mut Write = &mut cursored;

                for val in 0u8..255u8 {
                    writable_buffer.write_u8(val)?;
                }

            },
            _ => {

                return Err(anyhow!("Unsupported write method: {:?}", method));

            }
        }


    }

    info!("Allocated {:?} KB blocks in step of {:?} and wrote using method '{:?}'", &kb_blocks, &step, &method);

    Ok(my_vec)
}

use querystring;
fn handle_allocate_vector(_req: Request) -> Result<Response, Error> {
    info!("Handling Allocate Vector request");

    let query_string = _req.query_string().unwrap_or_default();
    let query_params = querystring::querify(&query_string);
    
    let mut param_hash: HashMap<&str,&str> = HashMap::new();
    for (k, v) in &query_params {
        param_hash.insert(k,v);
    }

    let kb_blocks: usize = param_hash["kb_blocks"].parse().expect("'kb_blocks' should be a valid integer");
    let step: usize = param_hash["step"].parse().expect("'step' should be a valid integer");
    let method: WriteMethod = WriteMethod::from_str(param_hash["write_method"]).expect("'write_method' not recognized");

    info!("Requested to allocate {} KB blocks in step of {} and using method {:?}. query_params: {:?}", &kb_blocks, &step, &method, &query_params);

    let test_vec = test_memory_allocation(kb_blocks, step, method).expect("Problem allocating test vector");

    let response = Response::new(200)
                    .body(Body::from(test_vec))
                    ;

    info!("Allocated test vector successfully!");
    // print_heap_info();

    Ok(response)
}

fn handle_memory_test(_req: Request) -> Result<Response, Error> {
    info!("Handling Memory Test request");

    test_memory_allocation(1024, 64, WriteMethod::Direct)?;

    let response = Response::new(200)
                    .body(Body::from("Memory test ran successfully!"))
                    ;

    info!("Memory test ran successfully!");

    Ok(response)
}


#[allow(unused_variables)]
fn httpd() -> Result<idf::Server> {
    let server = idf::ServerRegistry::new()
        .at("/")
        .get(|_| Ok("Hello, world!".into()))?
        .at("/mandelbrot")
        .get(handle_mandelbrot)?
        .at("/allocate_vector")
        .get(handle_allocate_vector)?
        .at("/memory_test")
        .get(handle_memory_test)?
        .at("/quit")
        .get(|_| bail!("Trying to quit..it may not be easy!"))?
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





