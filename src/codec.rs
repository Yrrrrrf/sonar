use std::error::Error;


// Core encoding/decoding methods
pub trait CodecTrait {
    // * Encode: bits -> signal
    fn encode(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>>;
    // * Decode: signal -> bits
    fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>>;
}

// // same as above but using some macro to reduce boilerplate...
// // * macro to reduce boilerplate
// macro_rules! impl_codec {
//     (
//         $($codec:ident),* $(,)?
//     ) => {
//         pub enum Codec {
//             $(
//                 $codec($codec),
//             )*
//         }

//         impl Codec {
//             pub fn encode(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>> {
//                 match self {
//                     $(
//                         Codec::$codec(codec) => codec.encode(data),
//                     )*
//                 }
//             }

//             pub fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>> {
//                 match self {
//                     $(
//                         Codec::$codec(codec) => codec.decode(samples),
//                     )*
//                 }
//             }
//         }
//     };
// }

// impl_codec!(
// add some modems to implement CodecTrait...
// );
