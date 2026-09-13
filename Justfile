# user_name        := env("USER")
# current_location := justfile()
# current_dir      := justfile_directory()
# module_name      := file_name(current_dir)

default: build

build:
    cargo build
    RUST_BACKTRACE=1 cargo test --features test-support
    cargo clippy

clean:
	cargo clean

cover:
	CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/coverage/cargo-test-%p-%m.profraw' cargo test
	grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html
	firefox target/coverage/html/index.html

