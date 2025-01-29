use std::time::Duration;

use dev_utils::{app_dt, dlog::*, format::*, info};
use sonar::{audio::{capture::AudioCapture, signal::SignalMonitor}, encoding::FSKEncoder};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    app_dt!(file!());
    set_max_level(Level::Trace);

    // Setup decoder with default FSK settings
    let capture = AudioCapture::default();

    info!("script::new({})", "AUDIO LISTENER".color(WHITE).style(Style::Bold));
    info!("Successfully started {} at {}", 
        "listening".color(GREEN), 
        dev_utils::datetime::DateTime::now().time
    );
    // Initialize signal monitor
    let mut monitor = SignalMonitor::new(48, Box::new(FSKEncoder::default()));
    monitor.print_header();

    let _ = capture.start_listening()?;

    // Main processing loop
    loop {
        std::thread::sleep(Duration::from_millis(100));
        let samples = capture.get_samples();
        monitor.process_samples(&samples);
    }
}
