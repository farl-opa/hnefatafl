# Hnefatafl Game Server

This project is a web-based server for playing various variants of the ancient Norse strategy board game Hnefatafl. The server supports multiple game modes and allows players to join and play games online or locally.

## Features

- Play different variants of Hnefatafl: Tablut, Brandubh, Hnefatafl, and Koch.
- Support for both local and online game modes.
- Real-time updates using Server-Sent Events (SSE).
- User authentication using session IDs stored in cookies.
- Dynamic HTML templates for rendering game boards and player lists.


## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (Rust package manager)

### Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/farl-opa/hnefatafl.git
    cd hnefatafl
    ```

2. Build the project:
    ```sh
    cargo build
    ```

3. Run the server:
    ```sh
    cargo run
    ```

### Usage

1. Open your web browser and navigate to [http://localhost:3030](http://_vscodecontentref_/13).
2. Enter your username to start a session.
3. Choose to play locally or online.
4. Select a game variant and start playing.

### Configuration

The server runs on port `3030` by default. You can change the port by modifying the [warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;](http://_vscodecontentref_/14) line in [main.rs](http://_vscodecontentref_/15).

### Project Modules

- [main.rs](http://_vscodecontentref_/16): The main entry point of the server, handles routing and session management.
- [brandubh.rs](http://_vscodecontentref_/17), [hnefatafl.rs](http://_vscodecontentref_/18), [koch.rs](http://_vscodecontentref_/19), [tablut.rs](http://_vscodecontentref_/20): Implementations of the different game variants.
- [templates](http://_vscodecontentref_/21): HTML templates for rendering the web pages.
- [images](http://_vscodecontentref_/22): Static assets for the game pieces and board.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgements

- [Warp](https://github.com/seanmonstar/warp) - The web framework used for the server.
- [Tokio](https://github.com/tokio-rs/tokio) - Asynchronous runtime for Rust.
- [Serde](https://github.com/serde-rs/serde) - Serialization framework for Rust.
- [Actix Web](https://github.com/actix/actix-web) - Web framework for Rust.

