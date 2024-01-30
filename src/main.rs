mod mock_alert_mgr;
mod mock_alert_rpt;
mod pcd04_msg;

fn main() {
    let alert_mgr_handle = std::thread::spawn(|| mock_alert_mgr::run_mock_alert_mgr());
    let alert_rpt_handle = std::thread::spawn(|| mock_alert_rpt::run_mock_alert_rpt());

    // Join the threads and handle the result
    thread_result(alert_mgr_handle.join(), "mock_alert_mgr");
    thread_result(alert_rpt_handle.join(), "mock_alert_rpt");
}

fn thread_result(result: std::thread::Result<()>, thread_name: &str) {
    match result {
        Ok(()) => println!("{} thread joined successfully.", thread_name),
        Err(err) => eprintln!("Error joining {} thread: {:?}", thread_name, err),
    }
}
