# Use Cases

Real-world applications for whatwg-html-rs.

## Web Scraping

Extract structured data from websites with proper HTML5 error recovery.

```rust
use whatwg_html_rs::parse;

let html = fetch_webpage("https://example.com/products");
let doc = parse(&html);

// Extract all product titles
for product in doc.root().query_all(".product-card h2").unwrap() {
    println!("Product: {}", product.text_content());
}

// Extract prices with attributes
for price in doc.root().query_all("[data-price]").unwrap() {
    if let Some(value) = price.attr("data-price") {
        println!("Price: ${}", value);
    }
}
```

## Content Sanitization

Clean user-generated HTML before storage or display. Prevents XSS attacks.

```rust
use whatwg_html_rs::parse;

// User submitted this potentially malicious HTML
let user_html = r#"
    <p>Hello!</p>
    <script>alert('xss')</script>
    <a href="javascript:steal()">Click me</a>
    <img src="x" onerror="evil()">
"#;

let doc = parse(user_html);
let safe_html = doc.to_html_safe();
// => <p>Hello!</p> <a>Click me</a> <img src="x">

// Scripts, javascript: URLs, and event handlers are removed
```

## Email HTML Processing

Safely render HTML emails by removing dangerous elements while preserving formatting.

```rust
use whatwg_html_rs::{parse, sanitize::{SanitizationPolicy, DEFAULT_POLICY}};

let email_html = fetch_email_body(message_id);
let doc = parse(&email_html);

// Use safe output - strips tracking pixels, scripts, external resources
let display_html = doc.to_html_safe();

// Or extract plain text for notifications
let preview = doc.to_text_safe();
println!("Email preview: {}...", &preview[..100.min(preview.len())]);
```

## HTML to Markdown Conversion

Convert HTML content to Markdown for static site generators or documentation.

```rust
use whatwg_html_rs::parse;

let html = r#"
    <h1>Welcome</h1>
    <p>This is a <strong>bold</strong> statement with a
       <a href="https://example.com">link</a>.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
    </ul>
"#;

let doc = parse(html);
let markdown = doc.to_markdown();
// # Welcome
//
// This is a **bold** statement with a [link](https://example.com).
//
// - Item 1
// - Item 2
```

## Static Site Generators

Process HTML templates with proper error recovery and transformation.

```rust
use whatwg_html_rs::parse;

let template = std::fs::read_to_string("template.html")?;
let doc = parse(&template);

// Find and modify elements
for meta in doc.root().query_all("meta[name='description']").unwrap() {
    // Process meta tags...
}

// Extract content sections
let main_content = doc.root().query_all("main").unwrap();
```

## Security Auditing

Analyze HTML for potentially malicious content.

```rust
use whatwg_html_rs::parse;

let html = fetch_page(url);
let doc = parse(&html);

// Find all scripts
let scripts = doc.root().query_all("script").unwrap();
println!("Found {} script tags", scripts.len());

// Find inline event handlers
let handlers = doc.root().query_all("[onclick], [onerror], [onload]").unwrap();
if !handlers.is_empty() {
    println!("Warning: {} inline event handlers found", handlers.len());
}

// Find external resources
let external = doc.root().query_all("[src^='http'], [href^='http']").unwrap();
for elem in external {
    println!("External resource: {:?}", elem.attr("src").or(elem.attr("href")));
}
```

## RSS/Atom Feed Processing

Parse and sanitize HTML content from RSS feeds.

```rust
use whatwg_html_rs::parse;

struct FeedItem {
    title: String,
    content_html: String,
    content_text: String,
}

fn process_feed_item(raw_html: &str) -> FeedItem {
    let doc = parse(raw_html);

    FeedItem {
        title: doc.root()
            .query_all("title")
            .unwrap()
            .first()
            .map(|t| t.text_content())
            .unwrap_or_default(),
        content_html: doc.to_html_safe(),
        content_text: doc.to_text_safe(),
    }
}
```

## CLI Usage

Process HTML from the command line:

```bash
# Parse and sanitize HTML file
whatwg-html input.html --safe > output.html

# Convert HTML to Markdown
whatwg-html input.html --format markdown > output.md

# Extract text content
whatwg-html input.html --format text

# Query specific elements
whatwg-html input.html --query "article p" --format text

# Process stdin
curl -s https://example.com | whatwg-html - --safe --format markdown

# Show parse errors
whatwg-html malformed.html --errors
```

## Batch Processing

Process multiple HTML files efficiently:

```rust
use whatwg_html_rs::parse;
use std::path::Path;

fn process_directory(dir: &Path) -> Vec<(String, String)> {
    let mut results = Vec::new();

    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e == "html").unwrap_or(false) {
            let html = std::fs::read_to_string(&path).unwrap();
            let doc = parse(&html);

            // Extract title and first paragraph
            let title = doc.root()
                .query_all("title")
                .unwrap()
                .first()
                .map(|t| t.text_content())
                .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string());

            let summary = doc.root()
                .query_all("p")
                .unwrap()
                .first()
                .map(|p| p.text_content())
                .unwrap_or_default();

            results.push((title, summary));
        }
    }

    results
}
```

## Performance Considerations

- **Parsing**: ~180ms for 10MB HTML
- **Serialization**: ~23ms for 10MB HTML
- **Memory**: Arena-based DOM minimizes allocations
- **Throughput**: 800,000+ small document ops/sec

For bulk processing, reuse parsed documents when possible rather than re-parsing.
