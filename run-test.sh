#!/usr/bin/sh

set -e

# this should be executed from inside the bash file and not from the Makefile
# because if this command runs inside the Makefile it gives an error "couldnd find screen"
xinit ./xinitrc -- $(command -v Xephyr) $TEST_SCREEN
