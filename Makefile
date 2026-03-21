.PHONY: all build daemon ui ui-debug extension test coverage clean dev run run-debug stop help

UI_RUN_ENV ?=
DAEMON_RUN_ENV ?=

all: build extension

build:
	cargo build

release:
	cargo build --release

daemon:
	cargo run -p daemon

ui:
	cargo run -p ui

ui-debug: UI_RUN_ENV=GTK_DEBUG=interactive GDK_DEBUG=events G_MESSAGES_DEBUG=all RUST_LOG=free_er_ui=debug
ui-debug:
	$(UI_RUN_ENV) cargo run -p ui

extension:
	cd extension && pnpm build

extension-watch:
	cd extension && pnpm watch

test:
	cargo test

coverage:
	@command -v cargo-llvm-cov >/dev/null 2>&1 || (echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; exit 1)
	@command -v jq >/dev/null 2>&1 || (echo "jq is required. Install it with your package manager (e.g. sudo apt install jq)."; exit 1)
	@command -v column >/dev/null 2>&1 || (echo "column is required (usually provided by util-linux/bsdextrautils)."; exit 1)
	@tmp_file="$$(mktemp)"; \
	cargo llvm-cov --workspace --all-features --json --summary-only --output-path "$$tmp_file" -- --test-threads=1; \
	jq -r '"File\tLines %\tRegions %\tFunctions %", (.data[0].files[] | "\(.filename)\t\(.summary.lines.percent // 0)\t\(.summary.regions.percent // 0)\t\(.summary.functions.percent // 0)"), "TOTAL\t\(.data[0].totals.lines.percent // 0)\t\(.data[0].totals.regions.percent // 0)\t\(.data[0].totals.functions.percent // 0)"' "$$tmp_file" | column -t -s "$$(printf '\t')"; \
	rm -f "$$tmp_file"

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
	@$(DAEMON_RUN_ENV) cargo run -p daemon &
	@sleep 1
	@echo "Starting UI..."
	@$(UI_RUN_ENV) cargo run -p ui

# Build everything (Rust + extension), then launch daemon + UI with GTK inspector enabled.
run-debug: DAEMON_RUN_ENV=RUST_LOG=free_er=debug
run-debug: UI_RUN_ENV=GTK_DEBUG=interactive GDK_DEBUG=events G_MESSAGES_DEBUG=all RUST_LOG=free_er_ui=debug
run-debug: run

clean:
	cargo clean
	rm -rf extension/dist

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "  run               Build everything and launch daemon + UI"
	@echo "                    Use UI_RUN_ENV='GTK_DEBUG=interactive GDK_DEBUG=events G_MESSAGES_DEBUG=all RUST_LOG=free_er_ui=debug' for UI debug logs"
	@echo "  build             Build all Rust crates (debug)"
	@echo "  release           Build all Rust crates (release)"
	@echo "  daemon            Run the daemon"
	@echo "  ui                Run the GTK4 UI"
	@echo "  ui-debug          Run GTK4 UI with inspector and click/event debug logging"
	@echo "  extension         Build the browser extension"
	@echo "  extension-watch   Watch and rebuild extension on changes"
	@echo "  stop              Kill any running daemon"
	@echo "  dev               Build + start daemon in background + launch UI"
	@echo "  test              Run all tests"
	@echo "  run-debug         Build everything and launch daemon + UI with inspector + click/event logs"
	@echo "  coverage          Generate HTML coverage report with cargo-llvm-cov"
	@echo "  coverage-summary  Print per-file and total coverage (lines/regions/functions) to stdout"
	@echo "  clean             Remove build artifacts and extension/dist"
	@echo "  help              Show this message"
