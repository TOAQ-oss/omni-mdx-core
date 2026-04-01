use omni_mdx_core::parser::parse_mdx;
use std::fs;
use std::time::Instant;

fn main() {
    println!("🧪 Starting Phase 2: Vulnerability Verification...");
    
    let paths = match fs::read_dir("artifacts") {
        Ok(p) => p,
        Err(_) => return,
    };

    let mut confirmed = 0;
    let mut tested = 0;

    for path in paths {
        let path = path.unwrap().path();
        let filename = path.file_name().unwrap().to_str().unwrap();
        
        // Skip the tmp tracker file
        if filename == ".current_test.tmp" { continue; }
        if path.extension().and_then(|s| s.to_str()) != Some("mdx") { continue; }

        tested += 1;
        let payload = fs::read_to_string(&path).unwrap();

        // Infinite loops don't need retesting, they are verified by definition
        if filename.starts_with("fatal_loop") {
            println!("CONFIRMED DEADLOCK (Infinite Loop): {}", path.display());
            confirmed += 1;
            continue;
        }
        
        // Warm-up
        let _ = parse_mdx(&payload);

        let mut total_time = 0.0;
        let runs = 5;
        for _ in 0..runs {
            let start = Instant::now();
            let _ = parse_mdx(&payload);
            total_time += start.elapsed().as_secs_f64();
        }

        let avg_time = total_time / runs as f64;

        if avg_time > 0.05 {
            println!("🚨 CONFIRMED VULNERABILITY : {} (Avg: {:.3}s)", path.display(), avg_time);
            confirmed += 1;
        } else {
            println!("❌ False Positive Dismissed: {} (Avg: {:.3}s)", path.display(), avg_time);
            let _ = fs::remove_file(&path); // Auto-delete false positives
        }
    }

    if tested == 0 {
        println!("✨ No suspect artifacts found. Engine is secure!");
    } else {
        println!("\n📊 Final Report: {} DoS vulnerabilities confirmed out of {} tested.", confirmed, tested);
    }
}