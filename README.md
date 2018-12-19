# Dipper
Dipper is a discord bot for checking and monitoring cryptocurrencies. 

# Usage
Before using Dipper please install all dependencies, including:
* MongoDB
* Rust
* Cargo

After that please start your instance of MongoDB and create `./conf/dipper.toml` modeled after the `./conf/dipper.toml.example` file.

Finally, build:
`cargo build --release`
 
 Then, run:
 `./target/release/dipper`
