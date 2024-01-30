use pcd_acm::mock_alert_rpt;

fn main() {
    let alert_rpt_handle = std::thread::spawn(|| mock_alert_rpt::run_mock_alert_rpt());

    if let Err(err) = alert_rpt_handle.join() {
        eprintln!("Error joining mock_alert_rpt thread: {:?}", err);
    }
}
