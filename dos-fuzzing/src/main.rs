//! Omni-Core Ultimate DoS-Fuzzer
//!
//! This diagnostic tool performs stress-testing and differential fuzzing
//! against the MDX parser and the OCP binary protocol.
//!
//! # Fuzzing Strategies
//! 1. **Russian Doll Patterns:** Generates deeply nested or repetitive structures
//!    to find O(n^2) or O(n!) complexity leaks.
//! 2. **Superlinear Scoring:** Measures execution time across increasing payload sizes
//!    to statistically detect algorithmic complexity vulnerabilities.
//! 3. **OCP Binary Fuzzing:** Mutates valid binary payloads and injects known
//!    edge cases (overflows, invalid opcodes) to test decoder robustness.
//! 4. **Isolation:** Runs each test in a monitored state to survive fatal hangs
//!    or infinite loops.

mod generators;
mod ocp;
mod scoring;
mod targets;

use generators::{generate_russian_doll, Pattern};
use ocp::{
    generate_deep_ocp, generate_empty_ocp, generate_flat_ocp, generate_overflow_attr_count,
    generate_overflow_root_count, generate_overflow_string_length, generate_random_valid_ocp,
    generate_unknown_opcode, mutate_ocp_payload,
};
use rand::Rng;
use targets::{measure_isolated, FuzzTarget};

use std::fs;
use std::time::Duration;

/// Maximum allowed time per individual test before it is flagged as suspicious.
const MAX_MILLIS: u64 = 500;
/// Fatal timeout — if a test exceeds this, the parser is considered frozen (Infinite Loop).
const FATAL_TIMEOUT: Duration = Duration::from_millis(MAX_MILLIS * 10);
/// Base size multiplier for incremental complexity testing.
const BASE_SIZE: usize = 100;

/// Tests an MDX text pattern for algorithmic complexity vulnerabilities.
fn test_mdx_pattern(pat: &Pattern, test_id: usize) -> bool {
    let mut time_samples = Vec::new();

    // Warmup phase: initializes lazy statics and caches.
    let warmup = format!(
        "{}{}{}",
        pat.prefix,
        pat.repeating.repeat(BASE_SIZE),
        pat.suffix
    );
    let _ = measure_isolated(FuzzTarget::MdxText(warmup), FATAL_TIMEOUT);

    for i in 1..=scoring::SAMPLE_SIZE {
        let size = BASE_SIZE * i;
        let payload = format!("{}{}{}", pat.prefix, pat.repeating.repeat(size), pat.suffix);

        let tmp_path = "artifacts/.current_test.tmp";
        let _ = fs::write(tmp_path, &payload);

        // Execute the target in isolation.
        match measure_isolated(FuzzTarget::MdxText(payload), FATAL_TIMEOUT) {
            Some(dur) => {
                time_samples.push((size as f64, dur.as_nanos() as f64));
                if dur.as_millis() as u64 > MAX_MILLIS {
                    break;
                }
            }
            None => {
                println!("\n  INFINITE LOOP — MDX text parser frozen.");
                let _ = fs::rename(
                    tmp_path,
                    format!("artifacts/fatal_loop_text_{}.mdx", test_id),
                );
                return true;
            }
        }
    }

    let _ = fs::remove_file("artifacts/.current_test.tmp");

    if time_samples.len() < 2 {
        return false;
    }

    // Statistical analysis: check if time increases significantly faster than payload size.
    let (score, is_vuln) = scoring::is_superlinear(&time_samples);
    let first = time_samples[0].1;
    let last = time_samples.last().unwrap().1;
    let expected_linear = first * scoring::SAMPLE_SIZE as f64;

    if is_vuln && last > expected_linear * 1.5 {
        let worst = format!(
            "{}{}{}",
            pat.prefix,
            pat.repeating.repeat(BASE_SIZE * scoring::SAMPLE_SIZE),
            pat.suffix
        );
        let _ = fs::write(format!("artifacts/suspect_mdx_{}.mdx", test_id), &worst);
        println!("   SUSPECT MDX — score: {:.2}", score);
        return true;
    }

    false
}

/// Tests the OCP Encoder/Decoder roundtrip for complexity leaks.
fn test_ocp_roundtrip(pat: &Pattern, test_id: usize) -> bool {
    let mut time_samples = Vec::new();

    let warmup = format!(
        "{}{}{}",
        pat.prefix,
        pat.repeating.repeat(BASE_SIZE),
        pat.suffix
    );
    let _ = measure_isolated(FuzzTarget::OcpRoundtrip(warmup), FATAL_TIMEOUT);

    for i in 1..=scoring::SAMPLE_SIZE {
        let size = BASE_SIZE * i;
        let payload = format!("{}{}{}", pat.prefix, pat.repeating.repeat(size), pat.suffix);

        match measure_isolated(FuzzTarget::OcpRoundtrip(payload), FATAL_TIMEOUT) {
            Some(dur) => {
                time_samples.push((size as f64, dur.as_nanos() as f64));
                if dur.as_millis() as u64 > MAX_MILLIS {
                    break;
                }
            }
            None => {
                println!("\n  INFINITE LOOP — OCP encoder frozen.");
                let worst = format!(
                    "{}{}{}",
                    pat.prefix,
                    pat.repeating.repeat(BASE_SIZE * scoring::SAMPLE_SIZE),
                    pat.suffix
                );
                let _ = fs::write(
                    format!("artifacts/fatal_loop_ocp_enc_{}.mdx", test_id),
                    &worst,
                );
                return true;
            }
        }
    }

    if time_samples.len() < 2 {
        return false;
    }

    let (score, is_vuln) = scoring::is_superlinear(&time_samples);
    let first = time_samples[0].1;
    let last = time_samples.last().unwrap().1;

    if is_vuln && last > first * scoring::SAMPLE_SIZE as f64 * 1.5 {
        let worst = format!(
            "{}{}{}",
            pat.prefix,
            pat.repeating.repeat(BASE_SIZE * scoring::SAMPLE_SIZE),
            pat.suffix
        );
        let _ = fs::write(format!("artifacts/suspect_ocp_enc_{}.mdx", test_id), &worst);
        println!("   SUSPECT OCP ENCODER — score: {:.2}", score);
        return true;
    }

    false
}

/// Injects malformed or mutated OCP binary payloads to test decoder resilience.
fn fuzz_ocp_binary(rng: &mut impl rand::Rng, iteration: usize) {
    // Known structural edge cases.
    let structured: Vec<(&str, Vec<u8>)> = vec![
        ("empty", generate_empty_ocp()),
        ("flat_10", generate_flat_ocp(10, "hello")),
        ("flat_1000", generate_flat_ocp(1000, "x")),
        ("deep_100", generate_deep_ocp(100)),
        ("deep_10000", generate_deep_ocp(10_000)),
        ("overflow_root_count", generate_overflow_root_count()),
        ("overflow_string_len", generate_overflow_string_length()),
        ("overflow_attr_count", generate_overflow_attr_count()),
        ("unknown_opcode", generate_unknown_opcode()),
    ];

    for (name, payload) in &structured {
        match measure_isolated(FuzzTarget::OcpBinary(payload.clone()), FATAL_TIMEOUT) {
            None => {
                println!("  INFINITE LOOP — OCP binary [{}]", name);
                let _ = fs::write(
                    format!("artifacts/fatal_loop_ocp_{}_{}.bin", name, iteration),
                    payload,
                );
            }
            Some(_) => {} // No hang → OK (panic = crash; in Rust, the thread terminates cleanly)
        }
    }

    // Random mutations of a valid payload.
    let base = generate_flat_ocp(5, "fuzz");
    let mutated = mutate_ocp_payload(rng, &base);

    match measure_isolated(FuzzTarget::OcpBinary(mutated.clone()), FATAL_TIMEOUT) {
        None => {
            println!("  INFINITE LOOP — OCP mutated binary");
            let _ = fs::write(
                format!("artifacts/fatal_loop_ocp_mut_{}.bin", iteration),
                &mutated,
            );
        }
        Some(_) => {}
    }

    // High depth valid payload.
    let depth = rng.gen_range(1..=20);
    let random_valid = generate_random_valid_ocp(rng, depth, 8);
    let _ = measure_isolated(FuzzTarget::OcpBinary(random_valid), FATAL_TIMEOUT);
}

fn main() {
    println!("  Starting Omni-Core Ultimate DoS-Fuzzer...");
    let _ = fs::create_dir_all("artifacts");
    let _ = fs::remove_file("artifacts/.current_test.tmp");

    let mut rng = rand::thread_rng();
    let mut i = 0usize;
    let mut suspects = 0usize;

    println!("\n=== Phase 1 : MDX Text + OCP Roundtrip Fuzzing ===");
    println!("  Press Ctrl+C to stop.\n");

    loop {
        i += 1;

        // Weighted probabilities for different fuzzing targets.
        let roll: f64 = rng.gen();

        if roll < 0.60 {
            let pat = generate_russian_doll(&mut rng, 4);
            if test_mdx_pattern(&pat, i) {
                suspects += 1;
            }
        } else if roll < 0.80 {
            let pat = generate_russian_doll(&mut rng, 4);
            if test_ocp_roundtrip(&pat, i) {
                suspects += 1;
            }
        } else {
            fuzz_ocp_binary(&mut rng, i);
        }

        // Progress indicator using Clippy's recommended is_multiple_of.
        if i.is_multiple_of(500) {
            println!(
                "... {} iterations — {} suspects saved to artifacts/",
                i, suspects
            );
        }
    }
}
