.PHONY: all build daemon ui extension test clean dev run stop help

all: build extension

build:
	cargo build

release:
	cargo build --release

daemon:
	cargo run -p daemon

ui:
	cargo run -p ui

extension:
	cd extension && pnpm build

extension-watch:
	cd extension && pnpm watch

test:
	cargo test

stop:
	@pkill -f "target/debug/free-er$$" 2>/dev/null || true
	@pkill -f "target/release/free-er$$" 2>/dev/null || true
	@rm -f /tmp/free-er.sock

# Run daemon in background, then launch UI
dev: build stop
	@echo "Starting daemon..."
	@cargo run -p daemon &
	@sleep 1
	@echo "Starting UI..."
	@cargo run -p ui

# Build everything (Rust + extension) and launch daemon + UI
run: build extension stop
	@echo "Starting daemon..."
	@cargo run -p daemon &
	@sleep 1
	@echo "Starting UI..."
	@cargo run -p ui

clean:
	cargo clean
	rm -rf extension/dist

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "  run               Build everything and launch daemon + UI"
	@echo "  build             Build all Rust crates (debug)"
	@echo "  release           Build all Rust crates (release)"
	@echo "  daemon            Run the daemon"
	@echo "  ui                Run the GTK4 UI"
	@echo "  extension         Build the browser extension"
	@echo "  extension-watch   Watch and rebuild extension on changes"
	@echo "  stop              Kill any running daemon"
	@echo "  dev               Build + start daemon in background + launch UI"
	@echo "  test              Run all tests"
	@echo "  clean             Remove build artifacts and extension/dist"
	@echo "  help              Show this message"
