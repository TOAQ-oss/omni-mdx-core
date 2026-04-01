use omni_mdx_core::binary::{decode_ast, encode_ast};
use omni_mdx_core::parser::parse_mdx;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Fuzz targets — each variant carries its own payload so the enum is
/// self-contained and can be moved into the spawned thread without lifetime issues.
pub enum FuzzTarget {
    /// Raw MDX text → parse_mdx
    MdxText(String),
    /// MDX text → parse_mdx → encode_ast (OCP roundtrip, no decoder needed yet)
    OcpRoundtrip(String),
    /// Raw corrupted bytes → future deserialize_ocp (stubbed until decoder exists)
    OcpBinary(Vec<u8>),
    /// encode → decode → re-encode → assert bytes equal (intégrité full roundtrip)
    OcpFullRoundtrip(String),
}

/// Runs the target in an isolated OS thread.
///
/// Returns `Some(duration)` on success, `None` if the thread hangs past `timeout`
/// (infinite loop / deadlock). The thread is intentionally leaked in that case —
/// the process will be killed by the OS when the fuzzer exits.
pub fn measure_isolated(target: FuzzTarget, timeout: Duration) -> Option<Duration> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let start = Instant::now();

        match target {
            FuzzTarget::MdxText(text) => {
                let _ = parse_mdx(&text);
            }

            FuzzTarget::OcpRoundtrip(text) => {
                // Full pipeline: parse MDX then encode to OCP binary.
                // Tests the encoder for panics/hangs on pathological ASTs.
                if let Ok(ast) = parse_mdx(&text) {
                    let _ = encode_ast(&ast);
                }
            }

            FuzzTarget::OcpBinary(bytes) => {
                let _ = decode_ast(&bytes);
            }

            FuzzTarget::OcpFullRoundtrip(text) => {
                if let Ok(ast) = parse_mdx(&text) {
                    let encoded = encode_ast(&ast);
                    if let Ok(decoded) = decode_ast(&encoded) {
                        let re_encoded = encode_ast(&decoded);
                        if encoded != re_encoded {
                            panic!("OCP roundtrip asymmetry detected");
                        }
                    }
                }
            }
        }

        let _ = tx.send(start.elapsed());
    });

    rx.recv_timeout(timeout).ok()
}