//! Omni-Core Performance Benchmark
//!
//! A minimal, standalone benchmarking tool to measure the end-to-end latency
//! of the `parse_mdx` and `compile_to_jsx` pipeline. It processes a heavily
//! nested MDX payload to simulate worst-case, real-world rendering scenarios.
//!
//! Note: While this provides a fast, CI-friendly latency check, for highly
//! rigorous statistical micro-benchmarking, the Rust community standard is
//! the `criterion` crate.

use omni_mdx_core::compiler::compile_to_jsx;
use omni_mdx_core::parser::parse_mdx;
use std::time::Instant;

fn main() {
    let complex_mdx = r#"
<ArticleLayout theme="dark" metadata={{ author: "Dr. Smith", date: "2026-03-14" }}>
  <Section id="methodology">
    # Methodology for the Vocal Dataset

    Here we detail the extraction process. Note the deep nesting below:

    <Callout type="warning" dismissible={true}>
      <Grid columns={2} gap="1rem">
        <Card shadow="lg">
          ### Phase 1: Isolation
          The isolation process requires separating the host voice from the guest voice.
          - Step A: Frequency analysis
          - Step B: Noise reduction
          
          <CodeBlock language="python">
            def extract_voice(audio_buffer):
                return model.process(audio_buffer)
          </CodeBlock>
        </Card>
      </Grid>
    </Callout>
  </Section>
</ArticleLayout>
"#;

    let iterations = 1000;
    println!(
        "🚀 Starting pure Rust benchmark ({} iterations)...",
        iterations
    );

    // Warm-up phase: Ensure any lazy static initializations (like the Regex pools
    // in the Markdown module) are compiled and cached before starting the timer.
    let _ = parse_mdx(complex_mdx);

    let start = Instant::now();

    for _ in 0..iterations {
        if let Ok(ast) = parse_mdx(complex_mdx) {
            let _jsx = compile_to_jsx(&ast);
        }
    }

    let duration = start.elapsed();
    let total_ms = duration.as_secs_f64() * 1000.0;

    println!("-----------------------------------");
    println!("⏱️ Total time : {:.2} ms", total_ms);
    println!(
        "⚡ Average per file : {:.4} ms",
        total_ms / iterations as f64
    );
    println!("-----------------------------------");
}
