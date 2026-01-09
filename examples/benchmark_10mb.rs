//! Performance benchmark with 10MB HTML file.

use whatwg_html_rs::{parse, selector::query_all};
use std::time::Instant;

fn generate_10mb_html() -> String {
    let mut html = String::with_capacity(11 * 1024 * 1024); // Pre-allocate
    html.push_str("<!DOCTYPE html>\n<html>\n<head><title>10MB Benchmark</title></head>\n<body>\n");

    // Each div with content is roughly 200 bytes
    // 10MB = 10,485,760 bytes
    // Need about 50,000 divs
    for i in 0..50000 {
        html.push_str(&format!(
            "<div class=\"item-{}\" id=\"id-{}\" data-index=\"{}\">\
            <p>Paragraph {} with some <b>bold</b> and <i>italic</i> text. \
            <a href=\"https://example.com/{}\">Link {}</a></p>\
            <ul><li>Item A</li><li>Item B</li><li>Item C</li></ul>\
            </div>\n",
            i, i, i, i, i, i
        ));
    }

    html.push_str("</body>\n</html>");
    html
}

fn benchmark_parse(html: &str, iterations: usize) -> f64 {
    // Warmup
    for _ in 0..2 {
        let _ = parse(html);
    }

    let mut times = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = parse(html);
        let elapsed = start.elapsed();
        times.push(elapsed.as_secs_f64() * 1000.0);
    }

    let avg = times.iter().sum::<f64>() / times.len() as f64;
    let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("Parse 10MB HTML:");
    println!("  Average: {:.2} ms", avg);
    println!("  Min: {:.2} ms", min);
    println!("  Max: {:.2} ms", max);

    avg
}

fn benchmark_serialize(html: &str, iterations: usize) -> f64 {
    let result = parse(html);

    // Warmup
    for _ in 0..2 {
        let _ = result.to_html();
    }

    let mut times = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = result.to_html();
        let elapsed = start.elapsed();
        times.push(elapsed.as_secs_f64() * 1000.0);
    }

    let avg = times.iter().sum::<f64>() / times.len() as f64;
    let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("Serialize 10MB HTML:");
    println!("  Average: {:.2} ms", avg);
    println!("  Min: {:.2} ms", min);
    println!("  Max: {:.2} ms", max);

    avg
}

fn benchmark_selector(html: &str, selector: &str, iterations: usize) -> f64 {
    let result = parse(html);

    // Warmup
    for _ in 0..2 {
        let _ = query_all(&result.dom, result.document, selector);
    }

    let mut times = Vec::with_capacity(iterations);
    let mut match_count = 0;

    for _ in 0..iterations {
        let start = Instant::now();
        let matches = query_all(&result.dom, result.document, selector).unwrap_or_default();
        let elapsed = start.elapsed();
        times.push(elapsed.as_secs_f64() * 1000.0);
        match_count = matches.len();
    }

    let avg = times.iter().sum::<f64>() / times.len() as f64;

    println!("Selector '{}': {} matches", selector, match_count);
    println!("  Average: {:.2} ms", avg);

    avg
}

fn main() {
    println!("{}", "=".repeat(60));
    println!("JustHTML Rust - 10MB HTML Benchmark");
    println!("{}", "=".repeat(60));
    println!();

    println!("Generating 10MB HTML...");
    let html = generate_10mb_html();
    let size_mb = html.len() as f64 / (1024.0 * 1024.0);
    println!("Generated {:.2} MB of HTML", size_mb);
    println!();

    println!("--- Benchmarks ---");
    println!();

    benchmark_parse(&html, 5);
    println!();

    benchmark_serialize(&html, 5);
    println!();

    println!("--- Selector Benchmarks ---");
    println!();

    benchmark_selector(&html, "div", 3);
    benchmark_selector(&html, ".item-25000", 3);
    benchmark_selector(&html, "div p b", 3);
    benchmark_selector(&html, "div:first-of-type", 3);

    println!();
    println!("{}", "=".repeat(60));
    println!("Benchmark complete");
    println!("{}", "=".repeat(60));
}
