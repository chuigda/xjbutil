.PHONY: all
all:
	echo "Please, run something else"

.PHONY: test
test:
	cargo test --package xjbutil -- --skip async_utils::test

.PHONY: test_async_tokio
test_async_tokio:
	cargo test --package xjbutil --lib async_utils::test

.PHONY: test_async_astd
test_async_astd:
	cargo test --package xjbutil --lib async_utils::test --no-default-features --features="enable-all async-astd"

.PHONY: test_async_pollster
test_async_pollster:
	cargo test --package xjbutil --lib async_utils::test --no-default-features --features="enable-all async-pollster"

.PHONY: miri_test
miri_test:
	MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --package xjbutil -- --skip async_utils::test

.PHONY: miri_test_async_tokio
miri_test_async_tokio:
	MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --package xjbutil --lib async_utils::test

.PHONY: miri_test_async_astd
miri_test_async_astd:
	MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --package xjbutil --lib async_utils::test \
		--no-default-features --features="enable-all async-astd"

.PHONY: miri_test_async_pollster
miri_test_async_pollster:
	MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --package xjbutil --lib async_utils::test \
		--no-default-features --features="enable-all async-pollster"
