set dotenv-load
build:
    CARGO_PROFILE_DEV_CODEGEN_BACKEND=cranelift cargo +nightly build -Zcodegen-backend
format:
    @cargo fmt --version
    cargo fmt
lint:
    @cargo clippy --version
    cargo clippy -- -D warnings -W clippy::pedantic -W clippy::nursery
    cargo doc
test:
    cargo nextest run --all-targets --no-fail-fast

t:test

build_lib:
    @cargo --version
    cargo build -p chip8-interpreter --release --features="rustler"
    cp ./target/release/libchip8_interpreter.dylib ./priv/libchip8_interpreter.so