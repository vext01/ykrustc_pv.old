#!/bin/sh
#
# Build script for continuous integration.

./x.py clean  # We don't clone afresh to save time and bandwidth.

# Note that the gdb must be Python enabled.
PATH=/opt/gdb-8.2/bin:${PATH} RUST_BACKTRACE=1 YK_DEBUG_SECTIONS=1 \
    ./x.py test --config .buildbot.toml
