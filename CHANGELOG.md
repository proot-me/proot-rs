# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2021-08-19
### Added
- Add support for path translation.
- Add support for multi-tracee.
- Add support for cross-compiling, including compiling to Android platform.
- Add unit test helper function test_with_proot() to enable testing proot-rs event loops in unit tests.
- Add integration tests.
- Add GitHub workflow scripts to automate testing and building release files.

### Changed
- Port loader.c to Rust
- Refactor errors.rs.
- Refactor executable loading process to support iterative loading.

### Fixed
- Fix the incorrect return value of system call.
- Fix the incorrect handling of trailing slash in paths during path translation.
- Fix the problem of incorrect handling of shebang.
- Fix the existing unit test function test_in_subprocess() so that it can report failed tests correctly.

[Unreleased]: https://github.com/proot-me/proot-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/proot-me/proot-rs/releases/tag/v0.1.0
