#!/usr/bin/env bash
# Verify that cargo-about is installed
if ! command -v "cargo-about"; then
    echo "Please install cargo-about:"
    echo "# cargo install cargo-about"
    exit 1
fi
# Create about.toml
cat <<EOT > about.toml
accepted = [
    "Apache-2.0",
    "MIT",
    "BSD-2-Clause",
]
EOT
# Initialize cargo-about
cargo-about init
# Generate LICENSES.html
EXIT_CODE=0
if ! cargo-about generate about.hbs > LICENSES.html; then
    echo "---------------------"
    echo -e "\033[0;31mSomething went wrong!\033[0m"
    echo "---------------------"
    EXIT_CODE=1
fi
# Cleanup
rm about.toml about.hbs
exit $EXIT_CODE