#!/bin/sh
#
# Build script for continuous integration.

./x.py clean  # We don't clone afresh to save time and bandwidth.

# Note that the gdb must be Python enabled.
#
# We are running a subset of tests for now since the gdb compile tests are
# currently failing due to out of data upstream code:
# https://github.com/softdevteam/ykrustc/pull/14#issuecomment-440608570
PATH=/opt/gdb-8.2/bin:${PATH} RUST_BACKTRACE=1 \
    ./x.py test --config .buildbot.toml \
    src/test/ui src/test/compile-fail src/test/run-pass
