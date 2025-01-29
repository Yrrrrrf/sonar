// * Morse code encoding and decoding
use std::fmt::Display;

// First, let's create a macro for the Signal enum and its basic implementations
macro_rules! define_signal {
    ($($variant:ident => $char:expr),* $(,)?) => {
        #[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
        pub enum Signal {$($variant),*}

        impl Signal {
            // todo: Remove this fn
            // todo: It's  redundant to the From<char> impl
            fn from_char(c: char) -> Option<Self> {
                match c {
                    $($char => Some(Signal::$variant),)*
                    _ => None,
                }
            }
        }

        impl From<char> for Signal {
            fn from(c: char) -> Self {
                match c {
                    $($char => Signal::$variant,)*
                    _ => Signal::CharGap, // Default case
                }
            }
        }

        impl From<Signal> for char {
            fn from(signal: Signal) -> Self {
                match signal {$(Signal::$variant => $char,)*}
            }
        }

        impl From<Signal> for String {
            fn from(signal: Signal) -> Self {char::from(signal).to_string()}
        }

        impl Display for Signal {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", char::from(*self))
            }
        }
    };
}

// A macro for defining morse patterns with automatic signal conversion
macro_rules! define_morse_patterns {
    ($($char:expr => $pattern:expr),* $(,)?) => {
        impl Morse {
            fn char_to_signal(c: char) -> Vec<Signal> {
                match c {
                    $($char => $pattern.chars().filter_map(Signal::from_char).collect(),)*
                    _ => vec![]
                }
            }

            fn signals_to_char(signals: &[Signal]) -> Option<char> {
                let pattern: String = signals.iter().map(|&s| char::from(s)).collect();
                match pattern.as_str() {
                    $($pattern => Some($char),)*
                    _ => None
                }
            }
        }
    };
}


// Usage example:
define_signal! {
    Dit => '.',
    Dah => '-',
    CharGap => ' ',
    WordGap => ' ',
}

pub struct Morse;

define_morse_patterns! {
    // &Special characters
    ' ' => " ",  // char gap 
    // &Letters
    'A' => ".-", 'B' => "-...", 'C' => "-.-.", 'D' => "-..",
    'E' => ".", 'F' => "..-.", 'G' => "--.", 'H' => "....",
    'I' => "..", 'J' => ".---", 'K' => "-.-", 'L' => ".-..", 'M' => "--", 'N' => "-.", 
    'O' => "---", 'P' => ".--.", 'Q' => "--.-", 'R' => ".-.", 'S' => "...", 'T' => "-",
    'U' => "..-", 'V' => "...-", 'W' => ".--", 'X' => "-..-", 'Y' => "-.--", 'Z' => "--..",
    // &Numbers
    '0' => "-----", '1' => ".----", '2' => "..---", '3' => "...--", '4' => "....-", 
    '5' => ".....", '6' => "-....", '7' => "--...", '8' => "---..", '9' => "----.",
}
// aceituna

// ^ (A) (C) (E) (I) (T) (U) (N) (A)
// ^ (.-) (.-.) (.-..) (.-..) (.-..) (.-..) (.-..) (.-..)
// ^ (00101110 00101101) (00101110 00101101 00101110)
// * on asqii: (char) -> Shift+(num)
// '.' -> 46 -> 0b00101110
// '-' on asqii -> 45 -> 0b00101101

impl Morse {
    /// Convert morse signals to text
    pub fn decode(signals: &[Signal]) -> String {
        let mut result = String::new();
        let mut current_char = Vec::new();
        let mut gap_count = 0;

        for &signal in signals {
            match signal {
                Signal::CharGap | Signal::WordGap => {
                    if !current_char.is_empty() {
                        if let Some(c) = Self::signals_to_char(&current_char) { result.push(c); }
                        current_char.clear();
                    }
                    if signal == Signal::WordGap && !result.ends_with(' ') && gap_count == 0 { 
                        result.push(' '); gap_count = 1; 
                    }
                },
                _ => { gap_count = 0; current_char.push(signal); }
            }
        }

        if !current_char.is_empty() {
            if let Some(c) = Self::signals_to_char(&current_char) { result.push(c); }
        }

        result.trim().to_string()
    }

    /// Convert text to morse signals
    pub fn encode(text: &str) -> Vec<Signal> {
        text.to_uppercase().chars().filter(|&c| c.is_alphanumeric() || c == ' ')
            .fold((Vec::new(), true), |(mut acc, first), c| {
                match c {
                    ' ' => { acc.push(Signal::WordGap); (acc, true) },
                    _ => {
                        if !first { acc.push(Signal::CharGap); }
                        acc.extend(Self::char_to_signal(c));
                        (acc, false)
                    }
                }
            }
        ).0  // select the signals (discard the boolean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_signals() {
        let s = Signal::Dit;
        let o = Signal::Dah;
        let c = Signal::CharGap;

        let signals = vec![s, s, s, c, o, o, o, c, s, s, s];  // S O S
        assert_eq!(Morse::decode(&signals), "SOS");
    }

    #[test]
    fn test_simple_words() {
        let input = "HELLO WORLD";
        let signals = Morse::encode(input);
        let output = Morse::decode(&signals);
        assert_eq!(output, input);
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(
            Morse::decode(&Morse::encode("H@LLO!")),
            "HLLO"
        );
        assert_eq!(Morse::decode(&Morse::encode("")), "");
        assert_eq!(Morse::decode(&Morse::encode("@#$%")), "");
    }

    #[test]
    fn test_space_handling() {
        assert_eq!(
            Morse::decode(&Morse::encode("A  B   C")),
            "A B C"
        );
        assert_eq!(
            Morse::decode(&Morse::encode("  SOS  ")),
            "SOS"
        );
    }

    #[test]
    fn test_char_gaps() {
        let signals = vec![
            Signal::Dit, Signal::Dit,  // I
            Signal::CharGap,
            Signal::Dit, Signal::Dah,  // A
            Signal::CharGap,
            Signal::Dah,  // T
        ];
        assert_eq!(Morse::decode(&signals), "IAT");
    }

    #[test]
    fn test_case_insensitivity() {
        let upper = "HELLO";
        let lower = "hello";
        let mixed = "HeLLo";
        
        let signals_upper = Morse::encode(upper);
        let signals_lower = Morse::encode(lower);
        let signals_mixed = Morse::encode(mixed);
        
        assert_eq!(Morse::decode(&signals_upper), "HELLO");
        assert_eq!(Morse::decode(&signals_lower), "HELLO");
        assert_eq!(Morse::decode(&signals_mixed), "HELLO");
    }

    #[test]
    fn test_alphanumeric() {
        let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let signals = Morse::encode(input);
        let output = Morse::decode(&signals);
        assert_eq!(output, input);
    }

    #[test]
    fn test_common_words() {
        let phrases = [
            "THE QUICK BROWN FOX",
            "JUMPS OVER THE LAZY DOG",
            "PACK MY BOX WITH FIVE DOZEN LIQUOR JUGS",
        ];

        for phrase in phrases {
            let signals = Morse::encode(phrase);
            let output = Morse::decode(&signals);
            assert_eq!(output, phrase);
        }
    }

    #[test]
    fn test_signal_patterns() {
        // Test a few specific Morse patterns
        let patterns = [
            ("SOS", "... --- ..."),
            ("OK", "--- -.-"),
            ("73", "--... ...--"),  // Common ham radio signals
        ];

        for (text, _pattern) in patterns {
            let signals = Morse::encode(text);
            let output = Morse::decode(&signals);
            assert_eq!(output, text);
        }
    }
}
