"""I337 fixed synthetic loopback fixture checks; no provider traffic."""
import json
import unittest
from unittest.mock import patch

from search_mcp_offline_observation import CASES, QUERY, STEPS, TOOLS, Reject, decode, observe

COMMIT = "9eee120a23cc446413a3db2e829f201b2aace7b0"
EXPECTED = {
    "json": "ok", "sse": "ok", "extra-tool": "ok", "schema-drift": "tool-schema",
    "duplicate-tool": "tool-schema", "missing-tool": "tool-schema", "bad-json": "malformed-json",
    "bad-sse": "malformed-sse", "wrong-id": "request-id", "rpc-error": "rpc-error",
    "tool-error": "tool-error", "bad-result": "result-schema", "partial": "partial",
    "oversized": "oversized", "redirect": "redirect", "401": "http-401", "403": "http-403",
    "429": "http-429", "500": "http-500", "proxy": "proxy", "tls": "tls",
    "cancelled": "cancelled", "deadline": "deadline", "budget": "budget", "invalid-url": "invalid-url",
}


class ObservationTests(unittest.TestCase):
    def test_all_provider_fixtures(self):
        self.assertEqual(set(CASES), set(EXPECTED))
        for provider in TOOLS:
            for case, outcome in EXPECTED.items():
                with self.subTest(provider=provider, case=case):
                    result = observe(provider, case, COMMIT)
                    self.assertEqual(result["outcome"], outcome)
                    self.assertEqual(result["region"], "LOCAL-ONLY")
                    self.assertFalse(result["vendor_observation"])
                    self.assertTrue(result["synthetic_only"])
                    self.assertLessEqual(result["requests"], result["max_requests"])
                    self.assertEqual(result["methods"], list(STEPS[:result["requests"]]))
                    if case in {"schema-drift", "duplicate-tool", "missing-tool", "proxy", "tls", "cancelled", "deadline", "budget"}:
                        self.assertEqual(result["requests"], 3)
                    if outcome == "ok":
                        self.assertEqual(result["admissible_count"], 1)
                        self.assertEqual(result["requests"], 4)
                    encoded = json.dumps(result)
                    for secret in (QUERY, "synthetic secret", "example.com", "example.invalid", "user:secret", "structuredContent"):
                        self.assertNotIn(secret, encoded)

    def test_ambient_proxy_does_not_change_loopback(self):
        with patch.dict("os.environ", {"HTTP_PROXY": "http://example.invalid:1", "ALL_PROXY": "http://example.invalid:1", "NO_PROXY": ""}):
            self.assertEqual(observe("exa", "json", COMMIT)["outcome"], "ok")

    def test_duplicate_id_and_boolean_rejected(self):
        for body in (b'{"jsonrpc":"2.0","id":1,"id":2,"result":{}}',
                     b'{"jsonrpc":"2.0","id":true,"result":{}}'):
            with self.assertRaises(Reject):
                decode(body, "application/json", 1)

    def test_invalid_metadata_rejected_before_network(self):
        with self.assertRaises(ValueError):
            observe("https://example.invalid", "json", COMMIT)
        with self.assertRaises(ValueError):
            observe("exa", "json", "not-a-commit")


if __name__ == "__main__":
    unittest.main()
