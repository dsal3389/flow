TEST_SCREEN=:1

test-release-build:
	cargo build --release && \
		TEST_SCREEN=$(TEST_SCREEN) sh ./run-test.sh
