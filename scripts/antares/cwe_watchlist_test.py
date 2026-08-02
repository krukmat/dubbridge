"""Tests for the versioned CWE watchlist (T3a)."""

import unittest

from scripts.antares.cwe_watchlist import (
    CweWatchlist,
    CweWatchlistEntry,
    WatchlistValidationError,
    load_watchlist,
    validate_entry,
)


class HappyPathTest(unittest.TestCase):
    def test_hp1_well_formed_entry_validates_and_is_retrievable_by_id(self):
        watchlist = load_watchlist()
        entry = watchlist.get("CWE-89")
        self.assertIsNotNone(entry)
        validate_entry(entry)  # does not raise
        self.assertEqual(entry.cwe_id, "CWE-89")
        self.assertTrue(entry.owner)
        self.assertTrue(entry.justification)
        self.assertTrue(entry.repository_boundary)

    def test_hp2_load_is_deterministic_and_version_stamped(self):
        first = load_watchlist()
        second = load_watchlist()
        self.assertEqual(first.cwe_ids(), second.cwe_ids())
        self.assertEqual(first.version, second.version)
        self.assertTrue(first.version)


class EdgeCaseTest(unittest.TestCase):
    def test_ec1_cwe_732_is_absent_by_construction(self):
        # CWE-732 (Incorrect Permission Assignment for Critical Resource) is
        # deliberately excluded: weak/overbroad class for current detection
        # precision, not an oversight -- see cwe_watchlist.py module docstring.
        watchlist = load_watchlist()
        self.assertNotIn("CWE-732", watchlist.cwe_ids())
        self.assertIsNone(watchlist.get("CWE-732"))

    def test_ec2_entry_missing_owner_fails_validation(self):
        malformed = CweWatchlistEntry(
            cwe_id="CWE-79",
            description="Cross-site Scripting",
            repository_boundary="apps/gateway/",
            owner="",
            justification="some justification",
        )
        with self.assertRaises(WatchlistValidationError):
            validate_entry(malformed)

    def test_ec2_entry_missing_justification_fails_validation(self):
        malformed = CweWatchlistEntry(
            cwe_id="CWE-79",
            description="Cross-site Scripting",
            repository_boundary="apps/gateway/",
            owner="security-team",
            justification="",
        )
        with self.assertRaises(WatchlistValidationError):
            validate_entry(malformed)

    def test_ec2_entry_missing_repository_boundary_fails_validation(self):
        malformed = CweWatchlistEntry(
            cwe_id="CWE-79",
            description="Cross-site Scripting",
            repository_boundary="",
            owner="security-team",
            justification="some justification",
        )
        with self.assertRaises(WatchlistValidationError):
            validate_entry(malformed)

    def test_ec3_duplicate_cwe_id_is_rejected_not_silently_shadowed(self):
        first = CweWatchlistEntry(
            cwe_id="CWE-89",
            description="SQL Injection",
            repository_boundary="crates/db/",
            owner="security-team",
            justification="justification one",
        )
        duplicate = CweWatchlistEntry(
            cwe_id="CWE-89",
            description="SQL Injection (duplicate)",
            repository_boundary="crates/db/",
            owner="security-team",
            justification="justification two",
        )
        with self.assertRaises(WatchlistValidationError):
            CweWatchlist("test-version", (first, duplicate))


if __name__ == "__main__":
    unittest.main()
