//! Performance benchmark for JustHTML Rust version.

use justhtml::{parse, selector::query_all};
use std::time::Instant;

const SMALL_HTML: &str = "<p>Hello, <b>World</b>!</p>";

const MEDIUM_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Test Page</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <header>
        <nav>
            <ul>
                <li><a href="/">Home</a></li>
                <li><a href="/about">About</a></li>
                <li><a href="/contact">Contact</a></li>
            </ul>
        </nav>
    </header>
    <main>
        <article>
            <h1>Welcome to Our Site</h1>
            <p>This is a paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
            <p>Another paragraph with a <a href="https://example.com">link</a>.</p>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
                <li>Item 3</li>
            </ul>
        </article>
    </main>
    <footer>
        <p>&copy; 2024 Test Company</p>
    </footer>
</body>
</html>
"#;

fn generate_large_html() -> String {
    let mut html = String::from(
        "<!DOCTYPE html>\n<html>\n<head><title>Large Document</title></head>\n<body>\n",
    );

    for i in 0..500 {
        html.push_str(&format!(
            "<div class=\"item-{}\"><p>Paragraph {} with some <b>bold</b> and <i>italic</i> text.</p></div>\n",
            i, i
        ));
    }

    html.push_str("</body>\n</html>");
    html
}

fn benchmark_parse(name: &str, html: &str, iterations: usize) -> f64 {
    // Warmup
    for _ in 0..10 {
        let result = parse(html);
        let _ = result.to_html();
    }

    let mut times = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let result = parse(html);
        let _ = result.to_html();
        let elapsed = start.elapsed();
        times.push(elapsed.as_secs_f64() * 1000.0); // ms
    }

    let avg: f64 = times.iter().sum::<f64>() / times.len() as f64;
    let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let variance: f64 = times.iter().map(|x| (x - avg).powi(2)).sum::<f64>() / times.len() as f64;
    let std_dev = variance.sqrt();

    let total_time_s: f64 = times.iter().sum::<f64>() / 1000.0;
    let throughput = iterations as f64 / total_time_s;

    println!("{}:", name);
    println!("  Iterations: {}", iterations);
    println!("  Average: {:.4} ms", avg);
    println!("  Std Dev: {:.4} ms", std_dev);
    println!("  Min: {:.4} ms", min);
    println!("  Max: {:.4} ms", max);
    println!("  Throughput: {:.0} ops/sec", throughput);
    println!();

    avg
}

fn benchmark_selector(html: &str, selector: &str, iterations: usize) -> f64 {
    let result = parse(html);

    // Warmup
    for _ in 0..10 {
        let _ = query_all(&result.dom, result.document, selector);
    }

    let mut times = Vec::with_capacity(iterations);
    let mut result_count = 0;

    for _ in 0..iterations {
        let start = Instant::now();
        let matches = query_all(&result.dom, result.document, selector).unwrap_or_default();
        let elapsed = start.elapsed();
        times.push(elapsed.as_secs_f64() * 1000.0);
        result_count = matches.len();
    }

    let avg: f64 = times.iter().sum::<f64>() / times.len() as f64;

    println!("Selector '{}':", selector);
    println!("  Average: {:.4} ms", avg);
    println!("  Results: {} matches", result_count);
    println!();

    avg
}

fn main() {
    println!("{}", "=".repeat(60));
    println!("JustHTML Rust Performance Benchmark");
    println!("{}", "=".repeat(60));
    println!();

    let large_html = generate_large_html();

    println!("--- Parsing Benchmarks ---");
    println!();

    benchmark_parse("Small HTML (parse + serialize)", SMALL_HTML, 5000);
    benchmark_parse("Medium HTML (parse + serialize)", MEDIUM_HTML, 1000);
    benchmark_parse("Large HTML (parse + serialize)", &large_html, 100);

    println!("--- Selector Benchmarks ---");
    println!();

    benchmark_selector(&large_html, "div", 1000);
    benchmark_selector(&large_html, ".item-250", 1000);
    benchmark_selector(&large_html, "div p b", 1000);

    println!("{}", "=".repeat(60));
    println!("Benchmark complete");
    println!("{}", "=".repeat(60));
}
