# user_name        := env("USER")
# current_location := justfile()
# current_dir      := justfile_directory()
# module_name      := file_name(current_dir)
target_dir       := `cargo metadata --no-deps --format-version=1 | jq -r '.target_directory'`

default: build

build:
    cargo build
    RUST_BACKTRACE=1 cargo test
    cargo clippy

clean:
	cargo clean

install-cover:
    cargo install cargo-llvm-cov

cover:
	cargo llvm-cov --html
	links2 {{target_dir}}/llvm-cov/html/index.html

