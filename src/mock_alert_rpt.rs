use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

use crate::pcd04_msg::PCD04Message;

#[allow(dead_code)]
struct MockAlertRpt {
    device_id: &'static str,
    device_location: &'static str,
    out_address: &'static str,
    out_part: u16,
}
#[allow(dead_code)]
impl MockAlertRpt {
    const DEVICE_ID: &'static str = "uuid:df041f5c-a3c9-11e9-8d8a-0050b612afeb";
    const DEVICE_LOCATION: &'static str = "POC^Room^Bed^fac^^^building^floor";
    const OUT_ADDRESS: &'static str = "127.0.0.1";
    const OUT_PORT: u16 = 8888;
    const SEND_HEARTBEAT: bool = true;
    fn send_alert() {
        print!("*** Sending Example Alert ***");

        let mut msg = PCD04Message::new();

        msg.create_pcd04_message(
            Self::DEVICE_ID,
            &format!("{}^^{}^URN", Self::DEVICE_ID, Self::DEVICE_ID),
            "HO2009001^^^Hospital^PI",
            "Abo^Nasser^^^L",
            "18991230",
            "M",
            "196670^MDC_EVT_LO^MDC",
            "Low Alert",
            "start",
            "PM",
            "1.1.1",
            "150456^MDC_PULS_OXIM_SAT_O2^MDC",
            "42",
            "NM",
            "",
            Self::DEVICE_ID,
            "SP",
            "",
            0,
            "",
            "",
            "",
            None,
            "",
            "69837^MDC_DEV_METER_PHYSIO_MULTI_PARAM_MDS^MDC",
            "69686^MDC_DEV_ANALY_BLD_CHEM_MULTI_PARAM_VMD^MDC",
        );
        if let Some(message) = msg.get_message() {
            let json_message = serde_json::to_string(&message);

            match json_message {
                Ok(json_str) => {
                    if let Ok(mut stream) = TcpStream::connect((Self::OUT_ADDRESS, Self::OUT_PORT))
                    {
                        if let Err(err) = stream.write_all(json_str.as_bytes()) {
                            eprintln!("Error writing to stream: {}", err);
                        }
                    } else {
                        eprintln!("Error connecting to server");
                    }
                }
                Err(err) => eprintln!("Error serializing message to JSON: {}", err),
            }
        }
    }

    fn create_heartbeat_msg() -> PCD04Message {
        let mut msg = PCD04Message::new();

        msg.create_pcd04_message(
            Self::DEVICE_LOCATION,
            &format!("{}^^{}^URN", Self::DEVICE_ID, Self::DEVICE_ID),
            "HO2009001^^^Hospital^PI",
            "Abo^Nasser^^^L",
            "18991230",
            "M",
            "196614^MDC_EVT_ACTIVE^MDC",
            "",
            "start",
            "PN",
            "1.1.1",
            "68480^MDC_ATTR_ALERT_SOURCE^MDC",
            "",
            "ST",
            "",
            Self::DEVICE_ID,
            "SA",
            "",
            0,
            "",
            "",
            "",
            None,
            "",
            "",
            "",
        );
        msg.append_watchdog_obx_segment("5", "None", "0.0.1");

        return msg;
    }

    fn main_loop(stop_event: Arc<Mutex<bool>>) {
        println!(
            "Opening socket to {}:{}",
            MockAlertRpt::OUT_ADDRESS,
            MockAlertRpt::OUT_PORT
        );

        if let Ok(mut out_socket) =
            TcpStream::connect((MockAlertRpt::OUT_ADDRESS, MockAlertRpt::OUT_PORT))
        {
            println!("Socket open, sending alive every 5 sec");
            let mut msg = Self::create_heartbeat_msg();

            loop {
                let msg_id = Uuid::new_v4().to_string();
                msg.set_control_id(&msg_id);
                println!("Sending msg with ID {}", msg_id);

                let json_message = serde_json::to_string(&msg.get_message().unwrap());

                match json_message {
                    Ok(json_str) => {
                        if let Err(err) = out_socket.write_all(json_str.as_bytes()) {
                            eprintln!("Error writing to stream: {}", err);
                        }
                    }
                    Err(err) => eprintln!("Error serializing message to JSON: {}", err),
                }

                thread::sleep(Duration::from_secs(1));

                if *stop_event.lock().unwrap() {
                    break;
                }
            }
        }
    }

    fn receive_loop(stop_event: Arc<Mutex<bool>>) {
        while !*stop_event.lock().unwrap() {
            if let Ok(mut out_socket) =
                TcpStream::connect((MockAlertRpt::OUT_ADDRESS, MockAlertRpt::OUT_PORT))
            {
                let mut received = [0; 1024 * 1024];
                match out_socket.read(&mut received) {
                    Ok(size) => {
                        if size > 0 {
                            let decoded = String::from_utf8_lossy(&received[..size]);
                            println!("Received message:\n{}\n", decoded);
                        }
                    }
                    Err(e) => eprintln!("Error reading from socket: {}", e),
                }
            } else {
                println!("Socket not open, next time...")
            }
        }
    }
}

pub fn run_mock_alert_rpt() {
    let stop_event = Arc::new(Mutex::new(false));
    let stop_event_clone = Arc::clone(&stop_event);

    ctrlc::set_handler(move || {
        *stop_event_clone.lock().unwrap() = true;
    })
    .expect("Error setting Ctrl+C");

    let stop_event1 = stop_event.clone();
    let main_handle = thread::spawn(move || {
        MockAlertRpt::main_loop(stop_event1.clone());
    });
    let stop_event_receive = Arc::clone(&stop_event);
    let receive_handle = thread::spawn(move || {
        MockAlertRpt::receive_loop(stop_event_receive);
    });

    println!("PCD-ACM AR Simulator");
    println!("Press a to Simulate sending an alert");
    println!("Press t to toggle heartbeat simulation");
    println!("Press q to quit");

    let mut next = true;
    while next {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let key = input.trim();
        match key {
            "q" => {
                next = false;
                let stop_event_clone = Arc::clone(&stop_event);
                *stop_event_clone.lock().unwrap() = true;
            }
            "a" => MockAlertRpt::send_alert(),
            "t" => {
                println!("Toggling heartbeat from {}", MockAlertRpt::SEND_HEARTBEAT);
            }
            _ => println!("Unknown key: {}", key),
        }
    }
    println!("Simulation completed...");

    main_handle.join().unwrap();
    receive_handle.join().unwrap();
}
