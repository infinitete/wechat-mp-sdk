# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-03-05

### Added

- Added watermark freshness checks via `Watermark::verify_timestamp_freshness`.
- Added `verify_watermark_with_max_skew` for configurable timestamp skew verification.
- Added deterministic jitter strategy for token refresh and retry backoff.

### Changed

- Refactored API token handling by centralizing token lifecycle and retry classification.
- Updated transient-error semantics: only network errors and HTTP `5xx/429` are treated as retryable.
- Updated `with_max_retries(0)` semantics to execute once without retry.
- Updated README dependency examples to `0.3`.

### Fixed

- Fixed potential wait hang in token single-flight refresh when a waiting caller is cancelled.
- Fixed binary media/qrcode response handling to prioritize HTTP status and robust `errcode/errmsg` detection.
- Fixed auth middleware logging to avoid exposing possibly sensitive token acquisition error details.

## [0.2.0] - 2026-02-22

### Changed

- Initial public release line after `v0.1.0`, including facade and endpoint coverage improvements.

