#!/usr/bin/env python3
"""Offline regressions for closed-Issue owner reconciliation."""

import sys
import unittest

sys.dont_write_bytecode = True
from validate_remote_issue_owners import owner_is_terminal


class TerminalOwnerTests(unittest.TestCase):
    def test_superseded_cancelled_owner_is_terminal(self):
        self.assertTrue(owner_is_terminal("# TASK-001\n\n**Status**: Cancelled / Superseded\n"))

    def test_plain_cancelled_owner_is_terminal(self):
        self.assertTrue(owner_is_terminal("**Status**: Cancelled\n"))

    def test_existing_completion_forms_remain_supported(self):
        for text in ("**Status**: Complete", "Status: Complete / Closed"):
            with self.subTest(text=text):
                self.assertTrue(owner_is_terminal(text))

    def test_active_or_historical_cancellation_is_not_terminal(self):
        for text in (
            "**Status**: Active\nCancellation remains planned.",
            "> **Status**: Cancelled / Superseded",
            "**Status**: Deferred\nThe previous task was Cancelled / Superseded.",
            "**Status**: CancelledLater",
        ):
            with self.subTest(text=text):
                self.assertFalse(owner_is_terminal(text))


if __name__ == "__main__":
    unittest.main()
