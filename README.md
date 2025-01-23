# V.A.P.O.R. 

**V**ysache's **A**utonomous **P**ropagation **O**bfuscation **R**outines  
*"Let your infrastructure quietly replicate itself."*

![Rust](https://img.shields.io/badge/Rust-1.67+-informational?logo=rust)
![Version](https://img.shields.io/badge/Version-0.1.0_Alpha-cyan)

A stealth-first C2 framework where infrastructure replication meets Rust-powered opsec.  

---

## Features ✨
- **Ephemeral Propagation**: Leave no disk artifacts through memory-resident agents
- **XOR Obfuscation Routines**: On-the-fly payload mutation to evade static analysis
- **Autonomous Replication**: Self-deploying agents via trusted process injection
- **TLS-Encrypted C2 Channels**: Mimics legitimate HTTPS traffic patterns
- **Plausible Deniability Architecture**: Agents self-terminate on forensic inspection

---

## Quick Start 🚀

### Installation
```bash
# Clone repo
git clone https://github.com/hexkaster/vapor-c2.git
cd vapor-c2

# Build with Cargo 
cargo build --release
```

run server:

```sh
./target/debug/server

```

compile client with the correct IP address:
```sh
   ____              _____ ___
  / __ \            / ____|__ \
 | |  | |_  ___   _| |       ) |
 | |  | \ \/ / | | | |      / /
 | |__| |>  <| |_| | |____ / /_
  \____//_/\_\\__, |\_____|____|
               __/ |
              |___/    - By @hexkaster

       Type 'help' for help.

[2024-09-07T17:49:10Z INFO  actix_server::builder] Starting 8 workers
[2024-09-07T17:49:10Z INFO  actix_server::builder] Starting "actix-web-service-0.0.0.0:8080" service on 0.0.0.0:8080
No previous history.
oxyc>> help
Available commands:
  sessions            - List all communicating beacons
  use [session index] - Use session
  help                - Display this help message
  exit                - Exit the current session
  cmd [command]       - Execute a shell command
  revshell            - Set up a reverse shell
  compile client [ip] - Compile client with specific IP address
oxyc>> compile client
Compiling client with IP: 192.168.15.8
Client compiled successfully with IP: 192.168.15.8
oxyc>>
```

run client:

```sh
./target/release/c2lient

```
