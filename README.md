# JustHTML (Rust)

A WHATWG HTML5 spec-compliant parser with sanitization and CSS selectors.

This is the Rust implementation of [JustHTML](https://github.com/EmilStenstrom/justhtml), providing **14-19x faster parsing** and **90-105x faster serialization** compared to the Python version.

## Features

- **HTML5 Compliant** - Implements the WHATWG HTML5 parsing specification with browser-grade error recovery
- **CSS Selectors** - Query the DOM with familiar CSS selector syntax
- **Sanitization** - Built-in XSS protection with configurable policies
- **Fast** - Parses 10MB HTML in ~180ms, serializes in ~23ms

## Usage

```rust
use justhtml::{parse, parse_fragment};

// Parse a full document
let doc = parse("<html><body><p>Hello!</p></body></html>");

// Parse a fragment (no <html>/<body> wrapper)
let frag = parse_fragment("<p><b>Hi</b></p>", "body");

// Query with CSS selectors
for node in doc.query("p").unwrap() {
    println!("{}", node.to_html());
}

// Sanitize HTML (removes scripts, dangerous attributes, etc.)
let clean = doc.sanitize(None);
println!("{}", clean.to_html());
```

## Sanitization

The sanitizer removes potentially dangerous content by default:

```rust
use justhtml::parse_fragment;

let doc = parse_fragment(
    r#"<p>Hello<script>alert(1)</script> <a href="javascript:alert(1)">bad</a></p>"#,
    "body"
);
let clean = doc.sanitize(None);
println!("{}", clean.to_html());
// => <p>Hello <a>bad</a></p>
```

## CSS Selectors

Supported selectors include:

- Type selectors: `div`, `p`, `*`
- Class selectors: `.class`
- ID selectors: `#id`
- Attribute selectors: `[attr]`, `[attr=value]`, `[attr^=prefix]`, `[attr$=suffix]`, `[attr*=contains]`
- Combinators: `A B` (descendant), `A > B` (child), `A + B` (adjacent), `A ~ B` (sibling)
- Pseudo-classes: `:first-child`, `:last-child`, `:nth-child()`, `:not()`, `:contains()`

## Performance

Benchmarks comparing Rust vs Python implementations:

| Operation | Python | Rust | Speedup |
|-----------|--------|------|---------|
| Parse 10MB | 3187 ms | 180 ms | **18x** |
| Serialize 10MB | 2686 ms | 23 ms | **115x** |
| Query `div` | 137 ms | 24 ms | **6x** |

See [BENCHMARK.md](../BENCHMARK.md) for detailed benchmarks.

## Optional GUI

A simple Iced-based GUI is available:

```bash
cargo run --release --features gui --bin justhtml-gui
```

## License

MIT
