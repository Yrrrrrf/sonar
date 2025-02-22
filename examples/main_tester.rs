#![allow(unused)] // silence unused warnings while developing

use std::time::Duration;

use dev_utils::{app_dt, debug, dlog::*, error, format::*, info, trace, warn};

// import some::*; from parent dir

// Example usage in main
fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);
}
