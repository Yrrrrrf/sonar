// * Some of the most useful dis-allows (to silence most of the clippy warnings)
// #![allow(unused)]

pub mod audio;
pub mod codec;
pub mod modem; 
#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "cli")]
fn start_cli() {
    println!("CLI mode enabled");
}

#[cfg(not(feature = "cli"))]
fn start_cli() {
    println!("CLI feature is not enabled.");
}

fn main() {
    start_cli();
}

#[cfg(test)]
mod tests {

    #[test]
    fn some_test() {
        assert_eq!((2_i32.pow(3)) - 4, 4);
    }

    #[test]
    fn some_test_2() {
        assert_eq!((2_i32.pow(3)) - 4, 4);
    }
}

// todo: FOR dev_utils: dev_macros/some(mod)
// todo:     - some custom proc-macro to gen:
// todo:         - new() w/ default & custom values
