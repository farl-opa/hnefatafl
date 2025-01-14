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

1. Open your web browser and navigate to [http://localhost:3030](http://localhost:3030).
2. Enter your username to start a session.
3. Choose to play locally or online.
4. Select a game variant and start playing.

### Configuration

The server runs on port `3030` by default. You can change the port by modifying the [warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;](http://_vscodecontentref_/14) line in [main.rs](https://github.com/farl-opa/hnefatafl/blob/master/src/main.rs).

### Project Modules

- [main.rs](https://github.com/farl-opa/hnefatafl/blob/master/src/main.rs): The main entry point of the server, handles routing and session management.
- [brandubh.rs](https://github.com/farl-opa/hnefatafl/blob/master/src/brandubh.rs), [hnefatafl.rs](https://github.com/farl-opa/hnefatafl/blob/master/src/hnefatafl.rs), [koch.rs](https://github.com/farl-opa/hnefatafl/blob/master/src/koch.rs), [tablut.rs](https://github.com/farl-opa/hnefatafl/blob/master/src/tablut.rs): Implementations of the different game variants.
- [templates](https://github.com/farl-opa/hnefatafl/tree/master/templates): HTML templates for rendering the web pages.
- [images](https://github.com/farl-opa/hnefatafl/tree/master/static/images): Static assets for the game pieces and board.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgements

- [Warp](https://github.com/seanmonstar/warp) - The web framework used for the server.
- [Tokio](https://github.com/tokio-rs/tokio) - Asynchronous runtime for Rust.
- [Serde](https://github.com/serde-rs/serde) - Serialization framework for Rust.
- [Actix Web](https://github.com/actix/actix-web) - Web framework for Rust.

