//! # Sovereign Fast File Finder — Standalone CLI Binary
//!
//! Run with: `cargo run -p rust_file_finder -- <folder_path> <query>`

use rust_file_finder::{FastDirectoryScanner, SmartAliasEngine};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    println!("=========================================================");
    println!("   ⚡ SOVEREIGN FAST FILE FINDER — 100% FREE & OPEN SOURCE");
    println!("   Sub-millisecond Rust Search & Smart In-Memory Alias");
    println!("=========================================================");

    let target_folder = if args.len() > 1 {
        args[1].clone()
    } else {
        "D:\\ragapp".to_string()
    };

    let query = if args.len() > 2 {
        args[2].clone()
    } else {
        "".to_string()
    };

    let path = PathBuf::from(&target_folder);
    if !path.exists() {
        println!("⚠️ Folder '{}' does not exist.", target_folder);
        return;
    }

    println!("🔍 Scanning scope: {}", target_folder);
    let scanner = FastDirectoryScanner::new();
    let alias_engine = SmartAliasEngine::new();

    let scan_start = Instant::now();
    let count = scanner.scan_path(&path);
    let scan_elapsed = scan_start.elapsed().as_secs_f64() * 1000.0;

    println!("✅ Indexed {} files in {:.2} ms", count, scan_elapsed);

    if !query.is_empty() {
        println!("\n🔎 Query: '{}'", query);
        let search_start = Instant::now();
        let results = scanner.search(&query);
        let search_elapsed = search_start.elapsed().as_secs_f64() * 1000.0;

        println!("🎯 Found {} matches in {:.3} ms\n", results.len(), search_elapsed);

        for (idx, record) in results.iter().take(10).enumerate() {
            let alias = alias_engine.resolve_alias(&record.path, None);
            println!("{}. {}", idx + 1, record.filename);
            println!("   Path: {}", record.path.display());
            println!("   Smart Alias: {}", alias.canonical_title);
            if alias.needs_renaming {
                if let Some(ref suggested) = alias.suggested_filename {
                    println!("   💡 Suggested Rename: {}", suggested);
                }
            }
            println!();
        }
    } else {
        println!("\nTip: Pass a query argument to search, e.g.:");
        println!("  cargo run -p rust_file_finder -- D:\\ragapp sandeep");
    }
}
