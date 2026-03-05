#!/bin/bash
# Helper script to run tests for the Rust Leaf client

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to check if server is running
check_server() {
    if curl -s http://leaf-server:5530 > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Change to leaf directory
cd "$(dirname "$0")/../.."

print_status "$YELLOW" "🌿 Leaf Client Rust Test Runner"
echo ""

# Check command line arguments
TEST_TYPE="${1:-unit}"

case "$TEST_TYPE" in
    "unit"|"u")
        print_status "$GREEN" "Running unit tests (no server required)..."
        echo ""
        cargo test -p leaf-client-rust --lib
        print_status "$GREEN" "✅ Unit tests passed!"
        ;;

    "integration"|"i")
        print_status "$YELLOW" "Running integration tests (requires server)..."
        echo ""

        if ! check_server; then
            print_status "$RED" "❌ Leaf server is not running at http://leaf-server:5530"
            echo ""
            echo "Start the server with:"
            echo "  cd leaf"
            echo "  cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token"
            echo ""
            echo "Or with Docker:"
            echo "  docker run -it --rm -p 5530:5530 -v \$(pwd)/data:/data leaf-server"
            exit 1
        fi

        print_status "$GREEN" "✅ Server is running"
        echo ""

        cargo test -p leaf-client-rust --test integration_test -- --ignored
        print_status "$GREEN" "✅ Integration tests passed!"
        ;;

    "all"|"a")
        print_status "$YELLOW" "Running all tests..."
        echo ""

        print_status "$GREEN" "Unit tests:"
        cargo test -p leaf-client-rust --lib
        echo ""

        if check_server; then
            print_status "$GREEN" "Integration tests:"
            cargo test -p leaf-client-rust --test integration_test -- --ignored
            print_status "$GREEN" "✅ All tests passed!"
        else
            print_status "$YELLOW" "⚠️  Skipping integration tests (server not running)"
            print_status "$GREEN" "✅ Unit tests passed!"
        fi
        ;;

    "bench"|"b")
        print_status "$YELLOW" "Running benchmarks..."
        echo ""
        cargo bench -p leaf-client-rust
        print_status "$GREEN" "✅ Benchmarks complete!"
        ;;

    "watch"|"w")
        print_status "$YELLOW" "Watching for changes and running tests..."
        echo ""
        if command -v cargo-watch &> /dev/null; then
            cargo watch -x "test -p leaf-client-rust"
        else
            print_status "$YELLOW" "cargo-watch not found. Install with:"
            echo "  cargo install cargo-watch"
            exit 1
        fi
        ;;

    "coverage"|"c")
        print_status "$YELLOW" "Generating test coverage..."
        echo ""
        if command -v cargo-tarpaulin &> /dev/null; then
            cargo tarpaulin -p leaf-client-rust --out Html
            print_status "$GREEN" "✅ Coverage report generated in html/index.html"
        else
            print_status "$YELLOW" "cargo-tarpaulin not found. Install with:"
            echo "  cargo install cargo-tarpaulin"
            exit 1
        fi
        ;;

    "doc"|"d")
        print_status "$YELLOW" "Generating and testing documentation..."
        echo ""
        cargo doc -p leaf-client-rust --no-deps
        cargo test -p leaf-client-rust --doc
        print_status "$GREEN" "✅ Documentation tests passed!"
        ;;

    "examples"|"e")
        print_status "$YELLOW" "Running examples..."
        echo ""
        print_status "$GREEN" "CBOR Demo:"
        cargo run -p leaf-client-rust --example cbor_demo
        echo ""
        print_status "$GREEN" "Client Demo:"
        cargo run -p leaf-client-rust --example client_demo
        print_status "$GREEN" "✅ Examples ran successfully!"
        ;;

    "check"|"ch")
        print_status "$YELLOW" "Checking compilation without running tests..."
        echo ""
        cargo check -p leaf-client-rust
        print_status "$GREEN" "✅ Code compiles successfully!"
        ;;

    "clippy"|"cl")
        print_status "$YELLOW" "Running Clippy linter..."
        echo ""
        cargo clippy -p leaf-client-rust -- -D warnings
        print_status "$GREEN" "✅ No Clippy warnings!"
        ;;

    "format"|"f")
        print_status "$YELLOW" "Formatting code..."
        echo ""
        cargo fmt -p leaf-client-rust -- --check
        print_status "$GREEN" "✅ Code is formatted correctly!"
        ;;

    "format-fix")
        print_status "$YELLOW" "Formatting code (fixing issues)..."
        echo ""
        cargo fmt -p leaf-client-rust
        print_status "$GREEN" "✅ Code formatted!"
        ;;

    *)
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  unit, u          Run unit tests (no server required)"
        echo "  integration, i   Run integration tests (requires server)"
        echo "  all, a           Run all tests"
        echo "  bench, b         Run benchmarks"
        echo "  watch, w         Watch for changes and run tests"
        echo "  coverage, c      Generate coverage report"
        echo "  doc, d           Run documentation tests"
        echo "  examples, e      Run examples"
        echo "  check, ch        Check compilation"
        echo "  clippy, cl       Run Clippy linter"
        echo "  format, f        Check code formatting"
        echo "  format-fix       Format code (fix issues)"
        echo ""
        echo "Examples:"
        echo "  $0 unit              # Run unit tests"
        echo "  $0 integration       # Run integration tests (requires server)"
        echo "  $0 all               # Run all tests"
        exit 1
        ;;
esac
