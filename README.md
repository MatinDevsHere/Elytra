# Elytra ![GitHub commit activity](https://img.shields.io/github/commit-activity/w/MatinDevsHere/Elytra?style=flat-square) ![GitHub last commit](https://img.shields.io/github/last-commit/MatinDevsHere/Elytra?style=flat-square) ![GitHub issues](https://img.shields.io/github/issues/MatinDevsHere/Elytra?style=flat-square) ![GitHub](https://img.shields.io/github/license/MatinDevsHere/Elytra?style=flat-square)

Elytra is a lightweight asynchronous server application written in Rust. It implements basic networking protocols
similar to those used in Minecraft, featuring packet handling (handshake, status, etc.) and an extensible logging
system.

## Features

- **Asynchronous Networking:** Uses Tokio for non-blocking I/O.
- **Packet Protocol:** Implements Minecraft-like packets including handshake and status response packets.
- **KISS:** Elytra is probably the simplest Minecraft Server Implementation you'll ever see. Including, but not limited
  to: macros, weird project structure, and webpack (excuse my sanity)

## State of the project

`undefined`. Elytra is so new that it even doesn't have a status yet. There's no way you can use it currently for
anything.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable version)
- Cargo (installed with Rust)

### Building the Project

Use Cargo to build the project:

```bash
cargo build --release
```

### Running the Server

To run the server, use the following command:

```bash
cargo run --release
```