use std::time::Instant;
use omni_mdx::parser::parse_mdx;
use omni_mdx::compiler::compile_to_jsx;

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
    println!("🚀 Lancement du benchmark Rust pur ({} itérations)...", iterations);

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
    println!("⚡ Average per file : {:.4} ms", total_ms / iterations as f64);
    println!("-----------------------------------");
}