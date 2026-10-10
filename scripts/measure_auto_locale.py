#!/usr/bin/env python3
"""Measure the checked-out production detector, without workspace dependencies.

This isolates detector CPU cost, not Agent/CLI latency or provider token usage.
Requires the repository-pinned rustc. No copied detector implementation is used.
"""
import argparse
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="Run pure production locale tests before measuring")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    source = (root / "crates/talos-agent/src/auto_resolver.rs").read_text()
    start = source.index("#[derive(Debug, Clone, PartialEq, Eq)]\nstruct ConversationLocale")
    end = source.index("/// Captures the directory object", start)
    rustc = os.environ.get("RUSTC", "rustc")
    version = subprocess.check_output([rustc, "--version"], text=True).strip()
    assert version.startswith("rustc 1.99.0 "), version
    harness = r'''
use std::hint::black_box;
use std::time::Instant;
const MAX_USER_INTENT_CHARS: usize = 4096;
'''+source[start:end]+r'''
fn main() {
    let fallback = ConversationLocale("en-US".into());
    let samples = ["请检查这个文件", "このファイルを確認してください",
                   "Please read the file and run the tests", "ls -la ./src"];
    for size in [32, 4096] {
        let window: Vec<String> = (0..8).map(|i| {
            samples[i % samples.len()].chars().cycle().take(size).collect()
        }).collect();
        for _ in 0..100 {
            for text in &window {
                black_box(ConversationLocale::detect_with_confidence(black_box(text), &fallback));
            }
        }
        let mut times = Vec::with_capacity(1000);
        for _ in 0..1000 {
            let started = Instant::now();
            for text in &window {
                black_box(ConversationLocale::detect_with_confidence(black_box(text), &fallback));
            }
            times.push(started.elapsed().as_nanos());
        }
        times.sort_unstable();
        println!("8-message window chars/message={size}: p50_ns={} p95_ns={}", times[500], times[950]);
    }
}
'''
    with tempfile.TemporaryDirectory(prefix="talos-locale-") as directory:
        src = Path(directory) / "measure.rs"
        binary = Path(directory) / "measure"
        src.write_text(harness)
        if args.self_test:
            names = [
                "conversation_locale_detection_is_bounded_and_deterministic",
                "conversation_locale_detection_reports_confidence_and_supported_scripts",
                "conversation_locale_detection_falls_back_for_mixed_or_unsupported_input",
                "history_selection_is_stable_across_repeated_approval_intents",
                "configured_locale_override_is_validated_and_presentation_only",
                "locale_evidence_reuses_detection_and_reset_discards_cache",
            ]
            tests = []
            for name in names:
                test_start = source.index("    fn " + name + "()")
                test_end = source.index("\n    }", test_start) + len("\n    }")
                tests.append("#[test]\n" + source[test_start:test_end])
            test_src = Path(directory) / "tests.rs"
            test_binary = Path(directory) / "tests"
            test_src.write_text(harness + "\n" + "\n".join(tests))
            subprocess.run([rustc, "--edition=2024", "--test", "-A", "dead_code", str(test_src), "-o", str(test_binary)], check=True)
            subprocess.run([str(test_binary)], check=True)
        subprocess.run([rustc, "--edition=2024", "-O", "-A", "dead_code", str(src), "-o", str(binary)], check=True)
        print(version, flush=True)
        subprocess.run([str(binary)], check=True)


if __name__ == "__main__":
    main()
