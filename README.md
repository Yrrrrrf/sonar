<h1 align="center">
    <img src="./resources/img/sonar.png" alt="Sonar Icon" width="128">
    <div>Sonar</div>
    <!-- <div><small>Audio-Based Data Transfer Protocol</small></div> -->
</h1>


[![GitHub](https://img.shields.io/badge/github-Yrrrrrf%2Fsonar-58A6FF?style=for-the-badge&logo=github)](https://github.com/Yrrrrrf/sonar)
[![Crates.io](https://img.shields.io/crates/v/sonar.svg?style=for-the-badge&logo=rust)](https://crates.io/crates/sonar)
[![Docs.rs](https://img.shields.io/badge/docs.rs-sonar-66c2a5?style=for-the-badge&labelColor=555555)](https://docs.rs/sonar)


## Features

- **Air-Gap Data Transfer**: Enable secure data transmission across air-gapped systems using audio signals
- **Real-time Signal Processing**: Monitor and visualize audio signals during transmission
- **Modular Encoding Support**: Flexible architecture supporting multiple encoding schemes (FSK, future: PSK, ASK)
- **Error Detection & Correction**: Built-in CRC and ECC for reliable data transfer
- **Cross-platform Compatibility**: Hardware-agnostic design working with standard audio devices
- **Configurable Parameters**: Adjustable frequency, sample rate, and transmission speed
- **Protocol Stack Architecture**: Layered design with frames, packets, and segments
- **Signal Strength Analysis**: Real-time monitoring of transmission quality

## Tech Stack

- **Core Protocol**:
    - [Rust](https://www.rust-lang.org/) for robust and efficient implementation
    - [cpal](https://crates.io/crates/cpal) for cross-platform audio I/O
    - [bytes](https://crates.io/crates/bytes) for efficient byte manipulation
- **Signal Processing**:
    - [rustfft](https://crates.io/crates/rustfft) for Fast Fourier Transform
    - Custom FSK implementation for digital encoding
- **Error Handling**:
    - Built-in CRC16 for error detection
    - Reed-Solomon ECC for error correction

## Protocol Stack

```text
┌─────────────────┐
│    Message      │ High-level container
├─────────────────┤
│     Frame       │ Transmission units
├─────────────────┤
│    Packet       │ Data organization
├─────────────────┤
│    Segment      │ Raw data handling
└─────────────────┘
```

## Installation

1. Clone the repository:
```bash
git clone https://github.com/Yrrrrrf/sonar.git
cd sonar
```

2. Build the project:
```bash
cargo build --release
```

3. Run tests:
```bash
cargo test
```

## Usage

### Basic Example
```rust
use sonar::{audio::{AudioDev, capture::AudioCapture, playback::AudioPlayback}, encoding::FSK};

// Initialize audio devices
let capture = AudioCapture::default();
let playback = AudioPlayback::new(Box::new(FSK::default()))?;
let device = AudioDev::new(capture, playback)?;

// Send data
let data = b"Hello, World!";
let stream = device.send(data)?;

// Receive data
let (stream, received) = device.listen()?;
```

### Signal Monitoring
```rust
use sonar::audio::signal::SignalMonitor;

let mut monitor = SignalMonitor::new(48, Box::new(FSK::default()));
monitor.print_header();
monitor.process_samples(&samples);
```

## Current Implementation Status

Our modular architecture includes:
- ✅ Core audio I/O system
- ✅ FSK encoding/decoding
- ✅ Frame-level protocol
- ✅ Basic error detection
- ✅ Signal monitoring
- 🔄 Advanced error correction
- 🔄 Flow control
- 🔄 Session management

## Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Commit changes: `git commit -m 'Add some feature'`
4. Push to branch: `git push origin feature/your-feature`
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
