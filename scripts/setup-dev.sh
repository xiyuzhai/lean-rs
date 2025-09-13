#!/bin/bash
# Development environment setup script for lean-claude

echo "Setting up lean-claude development environment..."

# Check if we're in a git repository
if [ ! -d .git ]; then
    echo "❌ Error: Not in a git repository. Please run from the project root."
    exit 1
fi

# Ensure git submodules are initialized
if [ -f .gitmodules ]; then
    echo "Ensuring git submodules are up-to-date..."
    git submodule update --init --recursive
    echo "✅ Submodules ready"
fi

# Create scripts directory if it doesn't exist
mkdir -p scripts

# Install git hooks
echo "Installing git hooks..."
if [ -f .git/hooks/pre-commit ] || [ -f .git/hooks/pre-push ]; then
    echo "Git hooks already exist. Backing up..."
    [ -f .git/hooks/pre-commit ] && mv .git/hooks/pre-commit .git/hooks/pre-commit.bak
    [ -f .git/hooks/pre-push ] && mv .git/hooks/pre-push .git/hooks/pre-push.bak
fi

# Create pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Pre-commit hook for lean-claude project

# Exit on any error
set -e

echo "Running pre-commit checks..."

# Ensure submodules are initialized (required for some tests)
if [ -f .gitmodules ]; then
    echo "Ensuring git submodules are up-to-date..."
    git submodule update --init --recursive
fi

# Check for unstaged changes
if ! git diff --quiet; then
    echo "❌ Error: You have unstaged changes. Please stage or stash them before committing."
    exit 1
fi

# Run cargo fmt check
echo "Checking formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Formatting check failed. Please run 'cargo fmt --all' before committing."
    exit 1
fi
echo "✅ Formatting check passed"

# Run clippy
echo "Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy check failed. Please fix all warnings before committing."
    exit 1
fi
echo "✅ Clippy check passed"

# Run tests
echo "Running tests..."
if ! cargo test --all; then
    echo "❌ Tests failed. Please fix all failing tests before committing."
    exit 1
fi
echo "✅ All tests passed"

echo "✅ All pre-commit checks passed!"
EOF

# Create pre-push hook
cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash
# Pre-push hook for lean-claude project

# Exit on any error
set -e

echo "Running pre-push checks..."

# Ensure submodules are initialized (required for some tests)
if [ -f .gitmodules ]; then
    echo "Ensuring git submodules are up-to-date..."
    git submodule update --init --recursive
fi

# Run cargo fmt check
echo "Checking formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Formatting check failed. Please run 'cargo fmt --all' before pushing."
    echo "To fix: cargo fmt --all"
    exit 1
fi
echo "✅ Formatting check passed"

# Run clippy
echo "Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy check failed. Please fix all warnings before pushing."
    exit 1
fi
echo "✅ Clippy check passed"

# Run tests
echo "Running tests..."
if ! cargo test --all; then
    echo "❌ Tests failed. Please fix all failing tests before pushing."
    exit 1
fi
echo "✅ All tests passed"

# Optional: Check that Cargo.lock is committed if Cargo.toml changed
if git diff --cached --name-only | grep -q "Cargo.toml"; then
    if ! git diff --cached --name-only | grep -q "Cargo.lock"; then
        echo "⚠️  Warning: Cargo.toml changed but Cargo.lock not staged."
        echo "Consider running 'cargo update' and committing Cargo.lock"
    fi
fi

echo "✅ All pre-push checks passed!"
EOF

chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push

echo "✅ Git hooks installed"

# Check Rust toolchain
echo "Checking Rust toolchain..."
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check for nightly toolchain (required for some features)
if ! rustup toolchain list | grep -q nightly; then
    echo "⚠️  Nightly toolchain not found. Installing..."
    rustup toolchain install nightly
fi

echo "✅ Rust toolchain ready"

# Install useful Rust tools
echo "Installing development tools..."

# Check and install cargo-watch for auto-rebuilding
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

# Check and install cargo-expand for macro debugging
if ! command -v cargo-expand &> /dev/null; then
    echo "Installing cargo-expand..."
    cargo install cargo-expand
fi

echo "✅ Development tools installed"

# Create convenience scripts
echo "Creating convenience scripts..."

# Create format script
cat > scripts/fmt.sh << 'EOF'
#!/bin/bash
echo "Running cargo fmt..."
cargo fmt --all
echo "✅ Code formatted"
EOF
chmod +x scripts/fmt.sh

# Create lint script
cat > scripts/lint.sh << 'EOF'
#!/bin/bash
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Linting complete"
EOF
chmod +x scripts/lint.sh

# Create test script
cat > scripts/test.sh << 'EOF'
#!/bin/bash
echo "Running all tests..."
cargo test --all
echo "✅ Tests complete"
EOF
chmod +x scripts/test.sh

# Create check-all script
cat > scripts/check-all.sh << 'EOF'
#!/bin/bash
set -e
echo "Running all checks..."
echo ""
./scripts/fmt.sh
echo ""
./scripts/lint.sh
echo ""
./scripts/test.sh
echo ""
echo "✅ All checks passed!"
EOF
chmod +x scripts/check-all.sh

echo "✅ Convenience scripts created in scripts/"

# Update gitignore to ignore backup files
if ! grep -q "*.bak" .gitignore 2>/dev/null; then
    echo "*.bak" >> .gitignore
fi

echo ""
echo "✅ Development environment setup complete!"
echo ""
echo "Available commands:"
echo "  ./scripts/fmt.sh       - Format all code"
echo "  ./scripts/lint.sh      - Run clippy linting"
echo "  ./scripts/test.sh      - Run all tests"
echo "  ./scripts/check-all.sh - Run all checks (format, lint, test)"
echo ""
echo "Git hooks installed:"
echo "  pre-commit - Runs format, clippy, and test checks before commit"
echo "  pre-push   - Runs format, clippy, and test checks before push"
echo ""
echo "Happy coding! 🎉"
