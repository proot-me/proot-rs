# proot-rust
Rust implementation of PRoot, a ptrace-based sandbox

# Requirements
Use the nightly Rust channel for rustc (``cargo default nightly``).

# Tests
Use ``RUST_TEST_THREADS=1`` before ``cargo test``, as a lot of tests are multi-thread,
and cargo runs them concurrently by default.