# CDArs
Continuous Double Auction Market implemented in Rust based for simulation purposes...

The purpose of this project is to act as a benchmark for comparing new market designs for decentralized exchanges. 

### Usage
By default the exchange accepts TCP connections on localhost:5000 and WebSocket connections via localhost:3015.
- Setup Rust: <https://www.rust-lang.org/tools/install>
- Make sure binary is compiled to your operating system, with "cargo build".
- Run "cargo run" in one terminal to start the CDA exchange server.
- Run "cargo run --example random_arrivals" in another terminal to start a simulation that sends random trader events (Enter, Update, and Cancel orders) to the exchange server. 

