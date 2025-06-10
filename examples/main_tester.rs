#![allow(unused)]

use dev_utils::{
    app_dt,
    dlog::*,
    format::{Style, Stylize},
};

fn main() {
    app_dt!(file!());
    // set_max_level(Level::Error);
    // set_max_level(Level::Warn);
    // set_max_level(Level::Info);
    set_max_level(Level::Debug);
    // set_max_level(Level::Trace);
}
