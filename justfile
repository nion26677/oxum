install:
    cargo build --release
    sudo cp target/release/oxum /usr/bin/oxum
