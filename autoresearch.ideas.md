# CodeMux Optimization Ideas

## Completed Optimizations

### 1. Remove regex dependency ✅
- **Status**: Complete - 55% binary size reduction achieved
- **Binary size**: 2048 KB → 916 KB
- **Approach**: Replaced 4 regex patterns with manual string operations
- **Date**: 2026-05-02

## Potential Future Optimizations

### 2. Further binary size reduction
- **Approach**: Use `panic = "abort"` in release profile
- **Expected savings**: Additional ~100-200 KB
- **Trade-off**: Removes panic stack traces (acceptable for CLI tool)

### 3. Use `codegen-units = 1`
- **Approach**: Add to profile.release for better LTO
- **Expected savings**: Small additional reduction
- **Trade-off**: Slower compile times

### 4. Optimize clap usage
- **Approach**: Use `default-features = false` and select only needed features
- **Expected savings**: Potentially 100-300 KB
- **Trade-off**: Need to verify --version and --help still work

### 5. Replace which crate
- **Approach**: Use `std::env::var("PATH")` and manual PATH traversal
- **Expected savings**: Small (~50 KB)
- **Trade-off**: More code to maintain

### 6. Replace dirs crate
- **Approach**: Use platform-specific env vars directly
- **Expected savings**: Small (~50 KB)
- **Trade-off**: Less portable, more platform-specific code

### 7. Startup time optimization
- **Approach**: Profile with `cargo flamegraph` to identify slow paths
- **Expected improvement**: Faster cold start for terminal integration
- **Measurement**: Use `hyperfine` to benchmark

## Notes
- The regex removal was the highest-impact optimization
- Current binary size (916 KB) is reasonable for a CLI tool
- Further optimizations have diminishing returns
