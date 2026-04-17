#!/usr/bin/env bash
# workspace.sh — convenience script for NovaForge Workspace
# Usage: ./workspace.sh <command>
#
# Commands:
#   run       Build and launch the NovaForge launcher
#   editors   Build and launch the full editor suite
#   build     Build all crates (debug)
#   release   Build all crates (optimised release)
#   build-game  Build the Nova-Forge game (requires submodule; see README)
#   test      Run all workspace tests
#   check     Run cargo check (fast compile check)
#   clippy    Run cargo clippy (lint)
#   clean     Remove build artefacts
#   help      Show this help message

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

CMD="${1:-help}"

case "$CMD" in
    run)
        echo "[workspace] Launching NovaForge Workspace launcher…"
        cargo run -p novaforge-workspace "${@:2}"
        ;;
    editors)
        echo "[workspace] Launching NovaForge Editor Suite…"
        cargo run -p novaforge-editors "${@:2}"
        ;;
    build)
        echo "[workspace] Building all crates (debug)…"
        cargo build --workspace "${@:2}"
        ;;
    release)
        echo "[workspace] Building all crates (release)…"
        cargo build --workspace --release "${@:2}"
        ;;
    build-game)
        echo "[workspace] Building Nova-Forge game (nova-forge/)…"
        if [ ! -d "$SCRIPT_DIR/nova-forge" ] || [ -z "$(ls -A "$SCRIPT_DIR/nova-forge")" ]; then
            echo "ERROR: nova-forge/ submodule not initialised."
            echo "Run: git submodule update --init --depth 1"
            exit 1
        fi
        cd "$SCRIPT_DIR/nova-forge"
        bash nova-forge.sh "${@:2}"
        ;;
    test)
        echo "[workspace] Running all tests…"
        cargo test --workspace "${@:2}"
        ;;
    check)
        echo "[workspace] Running cargo check…"
        cargo check --workspace "${@:2}"
        ;;
    clippy)
        echo "[workspace] Running clippy…"
        cargo clippy --workspace --all-targets -- -D warnings "${@:2}"
        ;;
    clean)
        echo "[workspace] Cleaning build artefacts…"
        cargo clean "${@:2}"
        ;;
    help|--help|-h)
        grep '^#' "$0" | sed 's/^# \?//'
        ;;
    *)
        echo "Unknown command: $CMD"
        echo "Run './workspace.sh help' for usage."
        exit 1
        ;;
esac
