# Elytra ![Static Badge](https://img.shields.io/badge/3k%2Fweek%20-%203k%2Fweek%20-%203k%2Fweek?style=flat-square&label=commit%20activity) ![GitHub last commit](https://img.shields.io/github/last-commit/MatinDevsHere/Elytra?style=flat-square) ![GitHub issues](https://img.shields.io/github/issues/MatinDevsHere/Elytra?style=flat-square) ![GitHub](https://img.shields.io/github/license/MatinDevsHere/Elytra?style=flat-square)

Elytra is a lightweight asynchronous server application written in Rust. It implements basic networking protocols
similar to those used in Minecraft, featuring packet handling (handshake, status, etc.) and an extensible logging
system.

## Features

- **Asynchronous Networking:** Uses Tokio for non-blocking I/O.
- **Packet Protocol:** Implements Minecraft-like packets including handshake and status response packets.
- **KISS:** Elytra is probably the simpelest Minecraft Server Implementation you'll ever see. Including, but not limited
  to: macros, weird project structure, and webpack (excuse my sanity)

## Why 1.16.5?
Because 1.8.9 is too old, and newer versions suck.

## State of the project

`undefined`. Elytra is so new that it even doesn't have a status yet. There's no way you can use it currently for
anything. However, I'm actively working on Elytra, so hopefully, we'll see a fork-and-serve-ready version of it soon.

## How to use it?

Elytra is not intended to be a vanila server replacement. Instead, Elytra focuses heavily on providing a solid and reliable (cough) Minecraft protocol foundation, so it can be forked and modified according to the community needs (e.g. making non-vanilla servrs such as Hypixel or MCCI).

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest nightly version)
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

## Did I intentionally leave words spelled wrong?

Good call! I try doing this regularly, so it will finally drive someone crazy and hopefully they open a PR to fix it and, `contributors++;`.
