.PHONY: build clean serve dev help install

# Default target
all: build

# Install dependencies
install:
	@echo "📦 Installing Rust WASM target..."
	rustup target add wasm32-unknown-unknown
	@echo "📦 Installing wasm-pack (if not already installed)..."
	@which wasm-pack > /dev/null || cargo install wasm-pack
	@echo "✅ Dependencies installed!"

# Build WASM module
build:
	@echo "🔨 Building WASM module..."
	wasm-pack build --target web --out-dir pkg
	@echo "✅ Build complete! Output in pkg/"

# Build with optimizations
build-release: build
	@echo "🚀 Build optimized for production"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf pkg/
	@echo "✅ Clean complete!"

# Start local development server
serve: build
	@echo "🌍 Starting local server on http://localhost:8000"
	@echo "📂 Open demo.html in your browser"
	@which python3 > /dev/null && python3 -m http.server 8000 || \
	(which python > /dev/null && python -m http.server 8000) || \
	@echo "⚠️  No Python found. Install Python or use another web server."

# Development mode with auto-rebuild (requires cargo-watch)
dev:
	@which cargo-watch > /dev/null || (echo "Installing cargo-watch..." && cargo install cargo-watch)
	@echo "👁️  Watching for changes..."
	cargo watch -s 'make build'

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Check code without building
check:
	@echo "🔍 Checking code..."
	cargo check

# Format code
fmt:
	@echo "✨ Formatting code..."
	cargo fmt

# Lint code
lint:
	@echo "📋 Linting code..."
	cargo clippy -- -D warnings

# Show help
help:
	@echo "EndlessUtopia - Makefile commands:"
	@echo ""
	@echo "  make install        - Install required dependencies"
	@echo "  make build          - Build WASM module"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make serve          - Build and start local web server"
	@echo "  make dev            - Watch mode with auto-rebuild"
	@echo "  make test           - Run tests"
	@echo "  make check          - Check code without building"
	@echo "  make fmt            - Format code"
	@echo "  make lint           - Lint code with clippy"
	@echo "  make help           - Show this help message"
	@echo ""
	@echo "Quick start: make serve"
