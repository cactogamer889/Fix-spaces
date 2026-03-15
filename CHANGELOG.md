# Changelog

## [2.0.0] - 2026-03-14

### Changed
- Rewrote scanner to use Windows native APIs (ctypes) instead of os.walk()
- Replaced threading with AsyncIO for better concurrency
- Added SQLite cache layer with mtime validation

### Added
- AsyncIO-based non-blocking disk scanning
- Persistent cache for faster rescans
- Performance metrics and logging
- Comprehensive test suite

### Performance
- 900GB scan: 45-60 seconds (was 5+ minutes)
- Rescans with cache: <5 seconds
- 8-12x faster enumeration with native APIs
