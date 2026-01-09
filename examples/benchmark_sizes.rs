//! Performance benchmark with various HTML file sizes.

use whatwg_html_rs::{parse, selector::query_all};
use std::time::Instant;

fn generate_html(target_kb: usize) -> String {
    let target_bytes = target_kb * 1024;
    let mut html = String::with_capacity(target_bytes + 1024);
    html.push_str("<!DOCTYPE html>\n<html>\n<head><title>Benchmark</title></head>\n<body>\n");

    let mut i = 0;
    while html.len() < target_bytes {
        html.push_str(&format!(
            "<div class=\"item-{}\" id=\"id-{}\" data-index=\"{}\">\
            <p>Paragraph {} with some <b>bold</b> and <i>italic</i> text. \
            <a href=\"https://example.com/{}\">Link {}</a></p>\
            <ul><li>Item A</li><li>Item B</li><li>Item C</li></ul>\
            </div>\n",
            i, i, i, i, i, i
        ));
        i += 1;
    }

    html.push_str("</body>\n</html>");
    html
}

fn benchmark_size(size_kb: usize) {
    let html = generate_html(size_kb);
    let actual_kb = html.len() / 1024;

    // Parse benchmark
    let mut parse_times = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        let _ = parse(&html);
        parse_times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let parse_avg = parse_times.iter().sum::<f64>() / parse_times.len() as f64;

    // Serialize benchmark
    let result = parse(&html);
    let mut serialize_times = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        let _ = result.to_html();
        serialize_times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let serialize_avg = serialize_times.iter().sum::<f64>() / serialize_times.len() as f64;

    // Selector benchmark
    let mut selector_times = Vec::new();
    let mut match_count = 0;
    for _ in 0..3 {
        let start = Instant::now();
        let matches = query_all(&result.dom, result.document, "div").unwrap_or_default();
        selector_times.push(start.elapsed().as_secs_f64() * 1000.0);
        match_count = matches.len();
    }
    let selector_avg = selector_times.iter().sum::<f64>() / selector_times.len() as f64;

    println!(
        "| {:>7} KB | {:>10.2} ms | {:>12.2} ms | {:>10.2} ms | {:>8} |",
        actual_kb, parse_avg, serialize_avg, selector_avg, match_count
    );
}

fn main() {
    println!("{}", "=".repeat(80));
    println!("JustHTML Rust - Multi-Size Benchmark");
    println!("{}", "=".repeat(80));
    println!();
    println!("| {:>10} | {:>13} | {:>14} | {:>12} | {:>8} |",
             "Size", "Parse", "Serialize", "Query 'div'", "Matches");
    println!("|{:-<12}|{:-<15}|{:-<16}|{:-<14}|{:-<10}|", "", "", "", "", "");

    let sizes = vec![50, 100, 200, 300, 400, 500, 1024, 5120, 10240];

    for size in sizes {
        benchmark_size(size);
    }

    println!();
    println!("{}", "=".repeat(80));
}
