use cpal::StreamConfig;



#[derive(Clone, Debug, Eq, PartialEq)]

pub struct AudioConfig { 
    config: StreamConfig,
        // pub channels: ChannelCount,
        // pub sample_rate: SampleRate,
        // pub buffer_size: BufferSize,

}
