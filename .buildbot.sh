#!/bin/sh
#
# Build script for continuous integration.

./x.py clean  # We don't clone afresh to save time and bandwidth.

# XXX Enable compiler tests once we have addressed:
# https://github.com/softdevteam/ykrustc/issues/10
RUST_BACKTRACE=1 ./x.py build --stage 1 src/libtest
