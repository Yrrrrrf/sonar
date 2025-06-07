<h1 align="center">
    <img src="https://raw.githubusercontent.com/Yrrrrrf/sonar/main/resources/img/sonar.png" alt="Sonar Icon" width="128">
    <div>Sonar</div>
</h1>

<div align="center">

[![GitHub](https://img.shields.io/badge/github-Yrrrrrf%2Fsonar-58A6FF?logo=github)](https://github.com/Yrrrrrf/sonar)
[![Crates.io](https://img.shields.io/crates/v/sonar.svg?logo=rust)](https://crates.io/crates/sonar)
[![Docs.rs](https://img.shields.io/badge/docs.rs-sonar-66c2a5)](https://docs.rs/sonar)
[![Crates.io Downloads](https://img.shields.io/crates/d/sonar)](https://crates.io/crates/sonar)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

</div>

Sonar is a Rust project exploring **data transmission using audio signals**. It aims to enable communication across air-gapped systems or in environments where traditional networking is not an option, using only standard microphones and speakers.

We're building a modular system with different sound encoding techniques (like FSK and BPSK) and a layered approach to structure the data for reliable transfer. The project includes a command-line tool to easily send and listen for these audio-based messages.

> **This project is currently in an active prototyping and development phase.** Many features are experimental and APIs are subject to change.

## Installation

```bash
cargo install sonar
```

## Examples

You can find more detailed examples in the [`examples/`](./examples/) directory:

*   **`main_tester.rs`**: Demonstrates the conceptual data structures for organizing information (Frames, Packets, Segments).
    ```bash
    cargo run --example main_tester
    ```
*   **`test.rs`**: Includes various smaller tests for different components like FSK configurations.
    ```bash
    cargo run --example test
    ```

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.