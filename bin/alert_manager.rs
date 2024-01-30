use pcd_acm::mock_alert_mgr;
fn main() {
    let alert_mgr_handle = std::thread::spawn(|| mock_alert_mgr::run_mock_alert_mgr());
    
    if let Err(err) = alert_mgr_handle.join() {
        eprintln!("Error joining mock_alert_mgr thread: {:?}", err);
    }
}
