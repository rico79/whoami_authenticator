# Prepare your dev environement (example for Fedora)

## Install Rust and tools
Install rust : [https://www.rust-lang.org/tools/install]

Install rust formatter
```
rustup component add rustfmt
```

Install cargo shuttle and login
```
cargo install cargo-shuttle
cargo shuttle login
```

## VSCode extensions
- rust-analyzer
- GitLab Workflow
- Even Better TOML

## To build and run the app
To run the app locally (localhost)
```
cargo shuttle run
```

To run the app exposed on local network (on 0.0.0.0:XXXX)
```
cargo shuttle run --external [--port XXXX]
```

To deploy the app
```
cargo shuttle deploy
```