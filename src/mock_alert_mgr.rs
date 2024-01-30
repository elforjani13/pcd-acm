use hl7::messages::ORU_R01;
use hl7::segments::OBX;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::SystemTime;

#[derive(Debug)]
struct MockAlertMgr;

impl MockAlertMgr {
    const START_MARKER: u8 = 0x0B;
    const END_MARKER: u8 = 0x1C;

    fn get_obx_segment<'a>(msgs: &'a [&OBX], id: &str) -> Option<&'a OBX> {
        msgs.iter()
            .find(|msg| msg.obx_4_observation_sub_id == id)
            .cloned()
    }

    fn receive_one_msg(stream: &mut TcpStream) -> Result<String, io::Error> {
        let mut start = [0];

        stream.read_exact(&mut start)?;

        if start[0] != MockAlertMgr::START_MARKER {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid start marker"));
        }

        let mut buffer = Vec::new();
        loop {
            let mut chunk = [0];
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(_) => {
                    buffer.push(chunk[0]);
                    if chunk[0] == MockAlertMgr::END_MARKER {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
        }

        Ok(String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
    }

    fn pretty(msg: &ORU_R01) -> String {
        let json_str = serde_json::to_string_pretty(msg).unwrap();
        json_str
    }

    fn send_acknowledgment(
        in_sock: &mut TcpStream,
        original_message_control_id: &str,
    ) -> Result<(), io::Error> {
        let acknowledgment_message = format!(
            "MSH|^~\\&|||||{}||ACK||P|2.1\rMSA|AA|{}\r",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            original_message_control_id
        );

        in_sock.write_all(&acknowledgment_message.into_bytes())?;
        Ok(())
    }
}

pub fn run_mock_alert_mgr() {
    println!("Binding socket...");
    let listener = TcpListener::bind("127.0.0.1:8888").unwrap();

    loop {
        println!("Waiting for connection...");
        let (mut in_sock, _) = match listener.accept() {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                continue;
            }
        };

        let msg = match MockAlertMgr::receive_one_msg(&mut in_sock) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                continue;
            }
        };

        match msg.parse::<ORU_R01>() {
            Ok(parsed_msg) => {
                let msg_type = parsed_msg.msh.msh_9_message_type.clone();
                let obs = &parsed_msg.oru_r01_patient_result[0]
                    .oru_r01_patient_observation
                    .iter()
                    .find(|obs| obs.obx.obx_4_observation_sub_id == "1")
                    .unwrap();
                let obx_slice = &[&obs.obx];

                if msg_type == "ACK^R41" {
                    println!("Got ACK {}", 9);
                } else if msg_type == "ORU^R40^ORU_R40" {
                    if let Some(seg) = MockAlertMgr::get_obx_segment(obx_slice, "1") {
                        let alarm_type_txt = seg.obx_3_observation_identifier.clone();
                        let mut answer = ORU_R01::default();

                        if alarm_type_txt == "196614^MDC_EVT_ACTIVE^MDC" {
                            println!("************ Got Heartbeat ************");
                            println!(
                                "Full message {}: \n{}",
                                SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                MockAlertMgr::pretty(&parsed_msg)
                            );

                            answer.msh.msh_9_message_type = "ACK^R40".to_string();
                            answer.msh.msh_15_accept_acknowledgment_type = Some("CA".to_string());

                            let orig_msg_ctrl_id = parsed_msg.msh.msh_10_message_control_id.clone();
                            if let Err(e) =
                                MockAlertMgr::send_acknowledgment(&mut in_sock, &orig_msg_ctrl_id)
                            {
                                eprintln!("Error sending acknowledgment: {}", e);
                                continue;
                            }
                        } else {
                            println!("************ Got Alarm {} ************", alarm_type_txt);
                            println!(
                                "Full message {}: \n{}",
                                SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                MockAlertMgr::pretty(&parsed_msg)
                            );

                            answer.msh.msh_9_message_type = "ACK^R40".to_string();
                            answer.msh.msh_4_sending_facility = Some("MockAM".to_string());

                            answer
                                .oru_r01_patient_result
                                .get_mut(0)
                                .and_then(|result| result.oru_r01_patient_observation.get_mut(0))
                                .and_then(|observation| {
                                    observation.prt.as_mut().map(|prt_vec| {
                                        if let Some(prt) = prt_vec.get_mut(0) {
                                            prt.prt_3_action_reason = Some("Delivered".to_string());
                                            "Delivered".to_string()
                                        } else {
                                            "Not Delivered".to_string()
                                        }
                                    })
                                })
                                .unwrap_or_else(|| "Not Delivered".to_string());

                            answer.msh.msh_10_message_control_id =
                                parsed_msg.msh.msh_10_message_control_id.clone();
                        }
                        println!(
                            "Answering {}: \n{:?}",
                            SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            MockAlertMgr::pretty(&answer)
                        );
                        if let Err(e) = in_sock.write_all(MockAlertMgr::pretty(&answer).as_bytes())
                        {
                            eprintln!("Error sending response: {}", e);
                            continue;
                        }
                        println!("*******************************");
                    }
                } else {
                    println!("Got unknown message type: {}", msg_type);
                }
            }

            Err(e) => {
                eprintln!("Error parsing message: {}", e);
            }
        }
    }
}

// pub fn run_mock_alert_mgr() {
//     println!("Binding socket...");
//     let listener = TcpListener::bind("127.0.0.1:8888").unwrap();
//
//     loop {
//         println!("Waiting for connection...");
//         let (mut in_sock, _) = listener.accept().unwrap();
//
//         let msg = MockAlertMgr::receive_one_msg(&mut in_sock).unwrap();
//
//         match msg.parse::<ORU_R01>() {
//             Ok(parsed_msg) => {
//                 let msg_type = parsed_msg.msh.msh_9_message_type.clone();
//                 let obs = &parsed_msg.oru_r01_patient_result[0]
//                     .oru_r01_patient_observation
//                     .iter()
//                     .find(|obs| obs.obx.obx_4_observation_sub_id == "1")
//                     .unwrap();
//                 let obx_slice = &[&obs.obx];
//
//                 if msg_type == "ACK^R41" {
//                     println!("Got ACK {}", 9);
//                 } else if msg_type == "ORU^R40^ORU_R40" {
//                     if let Some(seg) = MockAlertMgr::get_obx_segment(obx_slice, "1") {
//                         let alarm_type_txt = seg.obx_3_observation_identifier.clone();
//                         let mut answer = ORU_R01::default();
//
//                         if alarm_type_txt == "196614^MDC_EVT_ACTIVE^MDC" {
//                             println!("************ Got Heartbeat ************");
//                             println!(
//                                 "Full message {}: \n{}",
//                                 SystemTime::now()
//                                     .duration_since(SystemTime::UNIX_EPOCH)
//                                     .unwrap()
//                                     .as_secs(),
//                                 MockAlertMgr::pretty(&parsed_msg)
//                             );
//
//                             answer.msh.msh_9_message_type = "ACK^R40".to_string();
//                             answer.msh.msh_15_accept_acknowledgment_type = Some("CA".to_string());
//
//                             let orig_msg_ctrl_id = parsed_msg.msh.msh_10_message_control_id.clone();
//                             MockAlertMgr::send_acknowledgment(&mut in_sock, &orig_msg_ctrl_id);
//                         } else {
//                             println!("************ Got Alarm {} ************", alarm_type_txt);
//                             println!(
//                                 "Full message {}: \n{}",
//                                 SystemTime::now()
//                                     .duration_since(SystemTime::UNIX_EPOCH)
//                                     .unwrap()
//                                     .as_secs(),
//                                 MockAlertMgr::pretty(&parsed_msg)
//                             );
//
//                             answer.msh.msh_9_message_type = "ACK^R40".to_string();
//                             answer.msh.msh_4_sending_facility = Some("MockAM".to_string());
//
//                             answer
//                                 .oru_r01_patient_result
//                                 .get_mut(0)
//                                 .and_then(|result| result.oru_r01_patient_observation.get_mut(0))
//                                 .and_then(|observation| {
//                                     observation.prt.as_mut().map(|prt_vec| {
//                                         if let Some(prt) = prt_vec.get_mut(0) {
//                                             prt.prt_3_action_reason = Some("Delivered".to_string());
//                                             "Delivered".to_string()
//                                         } else {
//                                             "Not Delivered".to_string()
//                                         }
//                                     })
//                                 })
//                                 .unwrap_or_else(|| "Not Delivered".to_string());
//
//                             answer.msh.msh_10_message_control_id =
//                                 parsed_msg.msh.msh_10_message_control_id.clone();
//                         }
//                         println!(
//                             "Answering {}: \n{:?}",
//                             SystemTime::now()
//                                 .duration_since(SystemTime::UNIX_EPOCH)
//                                 .unwrap()
//                                 .as_secs(),
//                             MockAlertMgr::pretty(&answer)
//                         );
//                         in_sock
//                             .write_all(MockAlertMgr::pretty(&answer).as_bytes())
//                             .expect("Failed to send response");
//                         println!("*******************************");
//                     }
//                 } else {
//                     println!("Got unknown message type: {}", msg_type);
//                 }
//             }
//
//             Err(e) => {
//                 eprintln!("Error parsing message: {}", e);
//             }
//         }
//     }
// }
