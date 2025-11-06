#!/bin/bash

set -e

echo "Setting up SaaS Accelerator - Rust Edition"

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Rust version: $(rustc --version)"

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    echo "PostgreSQL is not installed. Please install PostgreSQL 14+"
    exit 1
fi

echo "PostgreSQL version: $(psql --version)"

# Check if Node.js is installed (for frontend)
if ! command -v node &> /dev/null; then
    echo "Node.js is not installed. Please install Node.js 18+ from https://nodejs.org/"
    exit 1
fi

echo "Node.js version: $(node --version)"

# Build Rust project
echo "Building Rust workspace..."
cargo build --release

# Setup frontend
echo "Setting up frontend..."
cd frontend
if [ ! -d "node_modules" ]; then
    npm install
fi
cd ..

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "Creating .env file from .env.example..."
    cp .env.example .env
    echo "Please edit .env file with your configuration"
fi

echo "Setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit .env file with your Azure and database credentials"
echo "2. Run database migrations: cd crates/data && sqlx migrate run"
echo "3. Start the services:"
echo "   - Admin API: cargo run --bin admin-api"
echo "   - Customer API: cargo run --bin customer-api"
echo "   - Webhook API: cargo run --bin webhook-api"
echo "   - Frontend: cd frontend && npm run dev"

