from __future__ import annotations

import argparse
import importlib.util
import json
import pathlib
import unittest

SCRIPT = pathlib.Path(__file__).with_name("search_provider_observation.py")
SPEC = importlib.util.spec_from_file_location("search_provider_observation", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
harness = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(harness)


class SearchProviderObservationTests(unittest.TestCase):
    def test_query_digest_contains_no_query_text(self) -> None:
        query = "sensitive 测试 query".encode()
        digest = harness.query_digest(query)
        self.assertEqual(digest["bytes"], len(query))
        self.assertEqual(len(digest["sha256"]), 64)
        self.assertNotIn("sensitive", str(digest))
        self.assertNotIn("测试", str(digest))

    def test_mwmbl_summary_separates_known_and_unknown_provenance(self) -> None:
        body = json.dumps(
            {
                "results": [
                    {"url": "https://example.invalid/a", "engine": "mwmbl"},
                    {"url": "https://example.invalid/b", "engine": "wikipedia"},
                    {"url": "https://example.invalid/c", "engine": "future"},
                    {"url": "mailto:x@example.invalid", "engine": "mwmbl"},
                ]
            }
        ).encode()
        summary = harness.summarize_json("mwmbl", body)
        self.assertEqual(summary["parse_class"], "valid-json")
        self.assertEqual(summary["wire_result_count"], 4)
        self.assertEqual(summary["admissible_http_url_count"], 3)
        self.assertEqual(
            summary["provenance_counts"],
            {"mwmbl": 2, "other": 1, "wikipedia": 1},
        )

    def test_purili_empty_and_error_envelopes_are_distinct(self) -> None:
        empty = harness.summarize_json("purili", b'{"results":[]}')
        error = harness.summarize_json("purili", b'{"error":"do not retain me"}')
        malformed = harness.summarize_json("purili", b"<html>challenge</html>")
        self.assertEqual(empty["parse_class"], "empty")
        self.assertEqual(error["parse_class"], "error-json")
        self.assertEqual(malformed["parse_class"], "malformed-json")
        self.assertNotIn("do not retain me", str(error))

    def test_admissible_url_rejects_credentials_and_non_http(self) -> None:
        self.assertTrue(harness.admissible_http_url("https://example.invalid/a"))
        self.assertTrue(harness.admissible_http_url("http://example.invalid/a"))
        self.assertFalse(harness.admissible_http_url("https://u:p@example.invalid/a"))
        self.assertFalse(harness.admissible_http_url("mailto:x@example.invalid"))
        self.assertFalse(harness.admissible_http_url("/relative"))

    def test_build_evidence_excludes_query_text_and_proxy_values(self) -> None:
        args = argparse.Namespace(
            provider="purili",
            talos_commit="a" * 40,
            region="Singapore",
            network_class="cloud",
            pair_id="pair-1",
            timeout_seconds=15.0,
        )
        query = b"private-query-text"
        evidence = harness.build_evidence(args, query, harness.failure_summary("dns"))
        rendered = json.dumps(evidence)
        self.assertNotIn("private-query-text", rendered)
        self.assertNotIn("HTTP_PROXY", rendered)
        self.assertEqual(evidence["operator_context"]["proxy_mode"], "direct")
        self.assertFalse(evidence["operator_context"]["geography_certified_by_harness"])

    def test_validation_requires_full_commit_and_bounded_pair_id(self) -> None:
        good = argparse.Namespace(
            talos_commit="b" * 40,
            pair_id="pair_2026-11-01",
            region="mainland China",
            network_class="residential",
            timeout_seconds=15.0,
        )
        harness.validate_metadata(good)

        bad_sha = argparse.Namespace(**vars(good))
        bad_sha.talos_commit = "abc"
        with self.assertRaises(ValueError):
            harness.validate_metadata(bad_sha)

        bad_pair = argparse.Namespace(**vars(good))
        bad_pair.pair_id = "contains space"
        with self.assertRaises(ValueError):
            harness.validate_metadata(bad_pair)

    def test_content_type_records_only_bounded_media_type(self) -> None:
        self.assertEqual(
            harness.bounded_content_type("application/json; charset=utf-8"),
            "application/json",
        )
        self.assertEqual(harness.bounded_content_type("unsafe value"), "other")


if __name__ == "__main__":
    unittest.main()
