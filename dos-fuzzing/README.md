# 🕵️ Omni-Core Ultimate Continuous DoS Fuzzer

This fuzzer is specifically designed to detect superlinear parsing behavior (Algorithmic Complexity Vulnerabilities / CWE-400), infinite loops, and Memory Exhaustion (OOM) within the `omni-mdx-core` engine. 

Patterns that exhibit $O(n^2)$ or worse execution time can be weaponized for Denial-of-Service (DoS) attacks, causing complete thread freezing and CPU exhaustion on host servers or WebAssembly clients.

## 🎯 Fuzzing Targets

Unlike traditional text-only fuzzers, the Omni-Core DoS Fuzzer operates on two distinct attack surfaces:
1. **MDX Parser (`MdxText`):** Tests highly nested, unclosed, or malformed combinations of Markdown, LaTeX, and JSX tags to detect catastrophic backtracking.
2. **OCP Binary Protocol (`OcpBinaryDeserialization`):** *(Planned)* Tests the resilience of the AST deserializer against forged, infinitely recursive, or corrupted binary payloads sent over the network.

## 🧬 Advanced Attack Vectors

To achieve 100% coverage against algorithmic vulnerabilities, this fuzzer implements enterprise-grade techniques:
* **Russian Doll Generation:** Patterns are built using a `prefix`, a `repeating` core, and a `suffix`. This reliably simulates unclosed AST nodes and infinite depth nesting.
* **Thread Isolation:** Every payload is executed in a disposable scoped thread with a strict hardware timeout. If the parser enters a deadlock/infinite loop, the thread is killed and the payload is flagged.
* **OOM (Out-of-Memory) Tracking:** Payloads are buffered to a `.tmp` file before execution. If the OS forcefully kills the fuzzer due to extreme RAM allocation, the exact payload is recovered automatically on the next run.

## 🧮 Mathematical Methodology

Instead of relying on simple, error-prone time thresholds, this fuzzer uses deterministic statistical mathematics to prove non-linearity.

1. The fuzzer repeats a random malicious pattern to generate $N$ samples of increasing byte-length.
2. It measures the parsing time for each sample, creating an array of `(length, time)` points.
3. It calculates the slope $\frac{\Delta y}{\Delta x}$ between *every possible pair of points*.
4. **The Proof:** If the parsing is linear $O(n)$, all slopes will be roughly identical. If the parsing is quadratic $O(n^2)$ or exponential, the slopes will diverge drastically. The fuzzer computes the **Standard Deviation** of these slopes. If it exceeds the `ACCEPTANCE_STDDEV` threshold, the pattern is mathematically proven to be superlinear.

## 🚀 Usage (Continuous Farm Workflow)

Do not run the binaries directly via `cargo`. This fuzzer is designed to run as a continuous background process (Fuzzing Farm), separating the discovery phase from the verification phase.

Make the scripts executable:
```bash
chmod +x run.sh retest.sh
```

### Phase 1: Continuous Discovery (`run.sh`)
Run this script in a terminal and leave it running in the background.
```bash
./run.sh
```
The fuzzer will run an infinite loop, generating thousands of patterns per minute. If the mathematical engine detects an anomaly, an infinite loop, or an OOM crash, it silently saves the exact payload inside the `artifacts/` directory and continues its hunt.

### Phase 2: On-Demand Verification (`retest.sh`)
Because modern OS schedulers and CPU thermal throttling can cause random latency spikes (False Positives), open a **new terminal** and run Phase 2 whenever you want to audit the collected artifacts.
```bash
./retest.sh
```
Each payload found in `artifacts/` is run 5 times in an isolated environment. False positives are automatically deleted from your disk. Only payloads that consistently choke the CPU are kept.

## 📊 Interpreting Results
At the end of the `retest.sh` run, the script will output a final report.

* **0 confirmed vulnerabilities:** Your parser and the Omni-Core Shield are perfectly calibrated.
* **`suspect_X.mdx`:** Confirmed $O(n^2)$ or exponential time complexity.
* **`fatal_loop_X.mdx`:** Confirmed infinite loop (deadlock). The parser never returned.
* **`fatal_oom_crash.mdx`:** Confirmed memory exhaustion. The payload forced Rust to allocate too much RAM.

Inspect the generated files in the `artifacts/` folder to debug the exact AST sequence that bypassed your defenses.

## 🚨 Reporting Vulnerabilities
If your fuzzing efforts uncover a confirmed vulnerability that successfully bypasses our defenses, **please do not open a public issue**. We take platform security very seriously and practice Responsible Disclosure. Please report your findings by:
1. Creating a draft advisory directly on our [GitHub Security Advisories](https://github.com/TOAQ-oss/omni-mdx-core/security/advisories) page.
2. Consulting our [SECURITY.md](/SECURITY.md) file at the root of the repository for detailed information on our disclosure policy, scope, and response timelines.