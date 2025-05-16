#!/bin/bash

# Update package lists
sudo apt-get update

# Install essential build tools
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev \
    git \
    curl

# Install Rust if not already installed
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Install cargo tools
cargo install cargo-make \
    cargo-watch \
    cargo-edit \
    cargo-audit \
    cargo-outdated

# Set up git hooks
chmod +x .git/hooks/pre-commit

# Create necessary directories
mkdir -p src tests docs/api docs/architecture docs/setup docs/contributing

echo "✅ Development environment setup complete!"
echo "Please run 'source $HOME/.cargo/env' to update your current shell with Rust environment variables." 