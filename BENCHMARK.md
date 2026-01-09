# Performance Benchmarks

Comprehensive benchmarks comparing implementations and parser strategies.

## Table of Contents
- [Python vs Rust](#python-vs-rust)
- [Rust Parser Comparison: State Machine vs Nom](#rust-parser-comparison-state-machine-vs-nom)

---

# Python vs Rust

Benchmark comparing the Rust state machine implementation and the Python JustHTML implementation.

## Test Environment

- **Date**: 2026-01-09
- **Platform**: macOS Darwin 24.0.0
- **Rust Version**: Release build with optimizations
- **Python Version**: 3.x with justhtml 0.30.0

## Parsing + Serialization Benchmarks

Tests measure the time to parse HTML and serialize it back to a string.

| Test | Python | Rust | Speedup |
|------|--------|------|---------|
| **Small HTML** | 0.0532 ms (18,802 ops/sec) | 0.0012 ms (833,674 ops/sec) | **44x faster** |
| **Medium HTML** | 0.3725 ms (2,685 ops/sec) | 0.0189 ms (52,848 ops/sec) | **20x faster** |
| **Large HTML** (500 divs) | 22.39 ms (45 ops/sec) | 0.80 ms (1,249 ops/sec) | **28x faster** |

### Test Documents

- **Small HTML**: `<p>Hello, <b>World</b>!</p>`
- **Medium HTML**: Full HTML5 document with header, nav, main, article, footer (~40 elements)
- **Large HTML**: Document with 500 div elements, each containing a paragraph with bold and italic text

## CSS Selector Benchmarks

Tests measure the time to query a pre-parsed document with CSS selectors.

| Selector | Python | Rust | Speedup |
|----------|--------|------|---------|
| `div` (500 matches) | 0.6819 ms | 0.0229 ms | **30x faster** |
| `.item-250` (1 match) | 1.7022 ms | 0.0247 ms | **69x faster** |
| `div p b` (500 matches) | 2.1827 ms | 0.0354 ms | **62x faster** |

## Scaling Benchmarks (Multi-Size)

Tests with HTML files from 50KB to 10MB to measure how performance scales.

### Parse Performance

| Size | Python | Rust | Speedup |
|------|--------|------|---------|
| 50 KB | 14.07 ms | 0.83 ms | **17x** |
| 100 KB | 28.18 ms | 1.62 ms | **17x** |
| 200 KB | 58.51 ms | 4.19 ms | **14x** |
| 300 KB | 89.17 ms | 5.75 ms | **16x** |
| 400 KB | 121.15 ms | 7.24 ms | **17x** |
| 500 KB | 181.89 ms | 9.53 ms | **19x** |
| 1 MB | 315.50 ms | 18.48 ms | **17x** |
| 5 MB | - | 118.00 ms | - |
| 10 MB | 3187.07 ms | 180.34 ms | **18x** |

### Serialize Performance

| Size | Python | Rust | Speedup |
|------|--------|------|---------|
| 50 KB | 8.08 ms | 0.09 ms | **90x** |
| 100 KB | 17.00 ms | 0.18 ms | **94x** |
| 200 KB | 36.88 ms | 0.41 ms | **90x** |
| 300 KB | 57.20 ms | 0.55 ms | **104x** |
| 400 KB | 73.98 ms | 0.75 ms | **99x** |
| 500 KB | 97.68 ms | 0.93 ms | **105x** |
| 1 MB | 212.92 ms | 2.04 ms | **104x** |
| 5 MB | - | 11.63 ms | - |
| 10 MB | 2685.82 ms | 23.35 ms | **115x** |

### Selector Query (`div`) Performance

| Size | Python | Rust | Speedup |
|------|--------|------|---------|
| 50 KB | 0.67 ms | 0.03 ms | **22x** |
| 100 KB | 1.33 ms | 0.06 ms | **22x** |
| 200 KB | 2.57 ms | 0.20 ms | **13x** |
| 300 KB | 3.88 ms | 0.44 ms | **9x** |
| 400 KB | 5.18 ms | 0.68 ms | **8x** |
| 500 KB | 6.36 ms | 0.82 ms | **8x** |
| 1 MB | 12.93 ms | 2.04 ms | **6x** |
| 5 MB | - | 12.21 ms | - |
| 10 MB | 137.23 ms | 23.99 ms | **6x** |

*Note: Python 5MB benchmark not run due to excessive time requirements.*

## Summary

The Rust implementation provides significant performance improvements across all benchmarks:

| Category | Speedup Range |
|----------|---------------|
| Parsing | **14-19x faster** |
| Serialization | **90-105x faster** |
| CSS Selector Queries | **6-69x faster** |

### Key Observations

1. **Serialization** shows the largest speedup (90-105x) across all file sizes
2. **Parsing** maintains consistent 14-19x speedup from 50KB to 1MB
3. **Selector queries** are extremely fast in Rust, especially for class selectors (69x)
4. **Large documents** (10MB): Rust parses in 180ms, serializes in 23ms
5. **Throughput** for small documents in Rust exceeds 800,000 operations per second

## Running the Benchmarks

### Rust Benchmark

```bash
cd justhtml-rs
cargo run --release --example benchmark
```

### Rust Multi-Size Benchmark

```bash
cd justhtml-rs
cargo run --release --example benchmark_sizes
```

### Python Benchmark

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -e .
python3 benchmark.py
```

### Python Multi-Size Benchmark

```bash
source .venv/bin/activate
python3 benchmark_sizes.py
```

## Test Coverage

### Python Tests
The Python implementation has **559 tests** covering all functionality.

### Rust Tests
The Rust implementation currently has **480 tests** (86% coverage of Python tests):

| Test Suite | Tests | Notes |
|------------|-------|-------|
| Selector | 119 | 1 ignored |
| Sanitizer | 85 | XSS prevention tests |
| Serialization | 81 | Round-trip tests |
| Integration | 47 | Parser behavior tests |
| Tokenizer | 45 | Token generation tests |
| Library | 98 | Core functionality |
| Doc-tests | 5 | API examples |

**Rust Total: 480 tests** (2 ignored for features not yet implemented)

### Features Not Yet in Rust
- Markdown conversion (`to_markdown()`)
- Some advanced sanitization options

---

# Rust Parser Comparison: State Machine vs Nom

Comparison of two tokenizer implementations in Rust.

## Overview

| Implementation | Branch | Lines of Code | Approach |
|----------------|--------|---------------|----------|
| State Machine | `release` | ~1,700 | Hand-written state machine per WHATWG spec |
| Nom | `nom` | ~500 | Declarative parser combinators |

## Parse Performance Comparison

| Size | State Machine | Nom | Difference |
|------|---------------|-----|------------|
| 50 KB | 0.89 ms | 1.05 ms | **1.18x slower** |
| 100 KB | 1.57 ms | 2.18 ms | **1.39x slower** |
| 200 KB | 3.99 ms | 5.62 ms | **1.41x slower** |
| 300 KB | 5.70 ms | 8.93 ms | **1.57x slower** |
| 400 KB | 7.78 ms | 12.14 ms | **1.56x slower** |
| 500 KB | 9.51 ms | 13.66 ms | **1.44x slower** |
| 1 MB | 18.20 ms | 25.95 ms | **1.43x slower** |
| 5 MB | 91.07 ms | 132.38 ms | **1.45x slower** |
| 10 MB | 176.82 ms | 252.39 ms | **1.43x slower** |

## Summary

| Parser | Avg Performance | 10MB Parse | Code Complexity |
|--------|-----------------|------------|-----------------|
| State Machine | **Baseline** | 177 ms | Higher (1,700 LOC) |
| Nom | ~1.4x slower | 252 ms | Lower (500 LOC) |

### Trade-offs

**State Machine (recommended for production):**
- ✅ ~40% faster parsing
- ✅ Lower memory allocations
- ✅ More precise WHATWG spec compliance
- ❌ Harder to read and maintain
- ❌ More prone to edge-case bugs

**Nom Parser:**
- ✅ Declarative, readable code
- ✅ Easier to modify and extend
- ✅ Fewer lines of code (3x reduction)
- ❌ ~40% slower than state machine
- ❌ Additional dependency

### Recommendation

Use the **state machine parser** (`release` branch) for production when performance matters. Use the **nom parser** (`nom` branch) for learning, prototyping, or when maintainability is prioritized over raw speed.

## Running the Comparison

```bash
# Benchmark state machine parser
git checkout release
cargo run --release --example benchmark_sizes

# Benchmark nom parser
git checkout nom
cargo run --release --example benchmark_sizes
```
