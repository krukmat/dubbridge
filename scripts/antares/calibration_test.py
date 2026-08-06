"""Unit tests for T4's ground-truth calibration metrics."""

from __future__ import annotations

import unittest

from scripts.antares.calibration import (
    CalibrationCase,
    CalibrationError,
    SnapshotRole,
    compute_calibration_report,
)


class HappyPathTest(unittest.TestCase):
    def test_hp1_precision_recall_file_f1_from_patch_derived_labels(self) -> None:
        case = CalibrationCase(
            case_id="v1",
            role=SnapshotRole.VULNERABLE,
            cwe_id="CWE-89",
            candidate_files=("crates/db/src/query.rs", "crates/db/src/pool.rs"),
            ground_truth_files=("crates/db/src/query.rs",),
        )
        report = compute_calibration_report((case,))
        self.assertEqual(len(report.task_metrics), 1)
        metrics = report.task_metrics[0]
        self.assertAlmostEqual(metrics.precision, 0.5)
        self.assertAlmostEqual(metrics.recall, 1.0)
        self.assertAlmostEqual(metrics.file_f1, 2 / 3)
        self.assertEqual(metrics.true_positives, 1)
        self.assertEqual(metrics.false_positives, 1)
        self.assertEqual(metrics.false_negatives, 0)

    def test_hp1_patched_snapshot_produces_true_negative_separately(self) -> None:
        patched = CalibrationCase(
            case_id="p1", role=SnapshotRole.PATCHED, cwe_id="CWE-89", candidate_files=()
        )
        report = compute_calibration_report((patched,))
        self.assertEqual(len(report.task_metrics), 0)
        self.assertEqual(len(report.true_negative_results), 1)
        self.assertTrue(report.true_negative_results[0].is_true_negative)
        self.assertEqual(report.true_negative_rate, 1.0)

    def test_hp1_vulnerable_and_patched_cases_never_pooled(self) -> None:
        vulnerable = CalibrationCase(
            case_id="v1",
            role=SnapshotRole.VULNERABLE,
            cwe_id="CWE-22",
            candidate_files=("crates/storage/src/key.rs",),
            ground_truth_files=("crates/storage/src/key.rs",),
        )
        patched = CalibrationCase(
            case_id="p1", role=SnapshotRole.PATCHED, cwe_id="CWE-22", candidate_files=("noise.rs",)
        )
        report = compute_calibration_report((vulnerable, patched))
        self.assertEqual(report.vulnerable_case_count, 1)
        self.assertEqual(report.patched_case_count, 1)
        self.assertEqual(report.true_negative_results[0].false_positive_files, ("noise.rs",))
        self.assertAlmostEqual(report.file_f1, 1.0)
        self.assertEqual(report.true_negative_rate, 0.0)

    def test_hp1_macro_average_is_unweighted_mean_of_per_case_scores(self) -> None:
        perfect = CalibrationCase(
            case_id="v1",
            role=SnapshotRole.VULNERABLE,
            cwe_id="CWE-89",
            candidate_files=("a.rs",),
            ground_truth_files=("a.rs",),
        )
        zero = CalibrationCase(
            case_id="v2",
            role=SnapshotRole.VULNERABLE,
            cwe_id="CWE-89",
            candidate_files=("b.rs",),
            ground_truth_files=("c.rs",),
        )
        report = compute_calibration_report((perfect, zero))
        self.assertAlmostEqual(report.mean_precision, 0.5)
        self.assertAlmostEqual(report.mean_recall, 0.5)
        self.assertAlmostEqual(report.file_f1, 0.5)


class EdgeCaseTest(unittest.TestCase):
    def test_ec_empty_case_list_raises(self) -> None:
        with self.assertRaises(CalibrationError) as ctx:
            compute_calibration_report(())
        self.assertEqual(ctx.exception.code, "empty_case_list")

    def test_ec_patched_case_with_ground_truth_raises(self) -> None:
        bad = CalibrationCase(
            case_id="p1",
            role=SnapshotRole.PATCHED,
            cwe_id="CWE-89",
            ground_truth_files=("a.rs",),
        )
        with self.assertRaises(CalibrationError) as ctx:
            compute_calibration_report((bad,))
        self.assertEqual(ctx.exception.code, "patched_case_has_ground_truth")

    def test_ec_vulnerable_case_missing_ground_truth_raises(self) -> None:
        bad = CalibrationCase(case_id="v1", role=SnapshotRole.VULNERABLE, cwe_id="CWE-89")
        with self.assertRaises(CalibrationError) as ctx:
            compute_calibration_report((bad,))
        self.assertEqual(ctx.exception.code, "vulnerable_case_missing_ground_truth")

    def test_ec_empty_case_id_raises(self) -> None:
        bad = CalibrationCase(
            case_id="  ",
            role=SnapshotRole.VULNERABLE,
            cwe_id="CWE-89",
            candidate_files=("a.rs",),
            ground_truth_files=("a.rs",),
        )
        with self.assertRaises(CalibrationError) as ctx:
            compute_calibration_report((bad,))
        self.assertEqual(ctx.exception.code, "empty_case_id")

    def test_ec_empty_cwe_id_raises(self) -> None:
        bad = CalibrationCase(
            case_id="v1",
            role=SnapshotRole.VULNERABLE,
            cwe_id="  ",
            candidate_files=("a.rs",),
            ground_truth_files=("a.rs",),
        )
        with self.assertRaises(CalibrationError) as ctx:
            compute_calibration_report((bad,))
        self.assertEqual(ctx.exception.code, "empty_cwe_id")

    def test_ec_duplicate_case_id_raises(self) -> None:
        case = CalibrationCase(
            case_id="v1",
            role=SnapshotRole.VULNERABLE,
            cwe_id="CWE-89",
            candidate_files=("a.rs",),
            ground_truth_files=("a.rs",),
        )
        with self.assertRaises(CalibrationError) as ctx:
            compute_calibration_report((case, case))
        self.assertEqual(ctx.exception.code, "duplicate_case_id")

    def test_ec_no_candidates_gives_zero_precision_not_division_error(self) -> None:
        case = CalibrationCase(
            case_id="v1",
            role=SnapshotRole.VULNERABLE,
            cwe_id="CWE-89",
            candidate_files=(),
            ground_truth_files=("a.rs",),
        )
        report = compute_calibration_report((case,))
        self.assertEqual(report.task_metrics[0].precision, 0.0)
        self.assertEqual(report.task_metrics[0].recall, 0.0)
        self.assertEqual(report.task_metrics[0].file_f1, 0.0)

    def test_ec_file_f1_never_used_as_per_output_probability(self) -> None:
        # File F1 is a macro-average over task_metrics; it must never carry
        # a per-case identity of its own, only be derived from the tuple.
        case = CalibrationCase(
            case_id="v1",
            role=SnapshotRole.VULNERABLE,
            cwe_id="CWE-89",
            candidate_files=("a.rs",),
            ground_truth_files=("a.rs",),
        )
        report = compute_calibration_report((case,))
        recomputed = sum(m.file_f1 for m in report.task_metrics) / len(report.task_metrics)
        self.assertAlmostEqual(report.file_f1, recomputed)


if __name__ == "__main__":
    unittest.main()
