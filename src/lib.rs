// * Some of the most useful dis-allows (to silence most of the clippy warnings)
#![allow(unused)]

pub mod audio;
pub mod codec;
pub mod modem;
pub mod stack;

// pub use modem::frame::{
//     Frame,

// };
// pub use modem::segment::Segment;

// #[cfg(feature = "cli")]

// #[cfg(feature = "cli")]
// fn start_cli() {
//     println!("CLI mode enabled");
// }

// #[cfg(not(feature = "cli"))]
// fn start_cli() {
//     println!("CLI feature is not enabled.");
// }

// fn main() {
//     start_cli();
// }

#[cfg(test)]
mod tests {

    #[test]
    fn some_test() {
        assert_eq!((2_i32.pow(3)) - 4, 4);
    }
}
