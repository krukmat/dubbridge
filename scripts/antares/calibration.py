"""Ground-truth calibration metrics (T4).

Computes task-level precision/recall/File F1 for Antares candidate-file
localization against patch-derived ground truth, plus a separate
true-negative metric for paired patched snapshots. This module never infers
per-output correctness from the aggregate metric it computes -- callers must
treat `CalibrationReport.file_f1` as a macro-average only (see
docs/tasks/antares-security-specialist-advisor.md T4 acceptance criteria).

Ground truth is always caller-supplied (typically extracted from a fixing
commit's changed-file list); this module never infers or fabricates it.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum


class CalibrationError(ValueError):
    """A fail-closed rejection with a stable machine-readable code."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


class SnapshotRole(Enum):
    """Whether a calibration case is a known-vulnerable pre-fix snapshot
    (positive case, ground truth = patch-derived vulnerable files) or a
    paired patched snapshot (negative case, ground truth is empty by
    definition -- any candidate is a false positive)."""

    VULNERABLE = "vulnerable"
    PATCHED = "patched"


@dataclass(frozen=True)
class CalibrationCase:
    """One calibration input: a snapshot, its role, the CWE under test, and
    the candidate files Antares actually returned for it.

    `ground_truth_files` is meaningful only for `SnapshotRole.VULNERABLE`
    cases (patch-derived vulnerable-file labels); for `SnapshotRole.PATCHED`
    cases it must be empty -- a patched snapshot has no vulnerable files by
    construction, and any non-empty candidate is a false positive for the
    true-negative metric, not a precision/recall input.
    """

    case_id: str
    role: SnapshotRole
    cwe_id: str
    candidate_files: tuple[str, ...] = field(default_factory=tuple)
    ground_truth_files: tuple[str, ...] = field(default_factory=tuple)


@dataclass(frozen=True)
class TaskMetrics:
    """Precision/recall/F1 for one `SnapshotRole.VULNERABLE` case."""

    case_id: str
    precision: float
    recall: float
    file_f1: float
    true_positives: int
    false_positives: int
    false_negatives: int


@dataclass(frozen=True)
class TrueNegativeResult:
    """The true-negative outcome for one `SnapshotRole.PATCHED` case."""

    case_id: str
    is_true_negative: bool
    false_positive_files: tuple[str, ...]


@dataclass(frozen=True)
class CalibrationReport:
    """Macro-averaged calibration results across every case supplied.

    `file_f1` (and `mean_precision`/`mean_recall`) are macro-averages over
    `task_metrics` -- an unweighted mean of per-case scores, not computed
    from pooled true/false-positive counts. Macro-averaging is the reporting
    contract this task's acceptance criteria require: do not infer
    per-output correctness from these aggregate numbers.
    """

    task_metrics: tuple[TaskMetrics, ...]
    true_negative_results: tuple[TrueNegativeResult, ...]
    mean_precision: float
    mean_recall: float
    file_f1: float
    true_negative_rate: float
    vulnerable_case_count: int
    patched_case_count: int


def _validate_case(case: CalibrationCase) -> None:
    if not case.case_id.strip():
        raise CalibrationError("empty_case_id", "CalibrationCase.case_id must be non-empty.")
    if not case.cwe_id.strip():
        raise CalibrationError(
            "empty_cwe_id", f"CalibrationCase {case.case_id!r} must carry a non-empty cwe_id."
        )
    if case.role is SnapshotRole.PATCHED and case.ground_truth_files:
        raise CalibrationError(
            "patched_case_has_ground_truth",
            f"CalibrationCase {case.case_id!r} is role=PATCHED but declares "
            "ground_truth_files; a patched snapshot has no vulnerable files by "
            "definition -- use SnapshotRole.VULNERABLE if this case has real "
            "patch-derived ground truth.",
        )
    if case.role is SnapshotRole.VULNERABLE and not case.ground_truth_files:
        raise CalibrationError(
            "vulnerable_case_missing_ground_truth",
            f"CalibrationCase {case.case_id!r} is role=VULNERABLE but declares no "
            "ground_truth_files; every vulnerable case requires patch-derived "
            "ground truth to compute precision/recall/File F1.",
        )


def _task_metrics_for(case: CalibrationCase) -> TaskMetrics:
    candidates = set(case.candidate_files)
    truth = set(case.ground_truth_files)
    true_positives = len(candidates & truth)
    false_positives = len(candidates - truth)
    false_negatives = len(truth - candidates)

    precision = true_positives / len(candidates) if candidates else 0.0
    recall = true_positives / len(truth) if truth else 0.0
    file_f1 = (
        (2 * precision * recall) / (precision + recall) if (precision + recall) > 0 else 0.0
    )
    return TaskMetrics(
        case_id=case.case_id,
        precision=precision,
        recall=recall,
        file_f1=file_f1,
        true_positives=true_positives,
        false_positives=false_positives,
        false_negatives=false_negatives,
    )


def _true_negative_for(case: CalibrationCase) -> TrueNegativeResult:
    false_positive_files = tuple(sorted(case.candidate_files))
    return TrueNegativeResult(
        case_id=case.case_id,
        is_true_negative=not case.candidate_files,
        false_positive_files=false_positive_files,
    )


def compute_calibration_report(cases: tuple[CalibrationCase, ...]) -> CalibrationReport:
    """Compute the macro-averaged calibration report over `cases`.

    HP-1: vulnerable cases contribute `TaskMetrics` (precision/recall/File
    F1); patched cases contribute a `TrueNegativeResult` instead -- the two
    populations are always evaluated separately and never pooled into one
    metric, matching the task's acceptance criteria that patched snapshots
    are "evaluated separately for true negatives."

    Raises `CalibrationError` (`empty_case_list`) if `cases` is empty --
    there is no meaningful macro-average over zero cases, and silently
    returning a zero-valued report would misrepresent an empty calibration
    run as a calibration run that scored zero.
    """
    if not cases:
        raise CalibrationError("empty_case_list", "compute_calibration_report requires at least one case.")

    seen_ids: set[str] = set()
    for case in cases:
        _validate_case(case)
        if case.case_id in seen_ids:
            raise CalibrationError(
                "duplicate_case_id", f"case_id {case.case_id!r} appears more than once."
            )
        seen_ids.add(case.case_id)

    task_metrics = tuple(
        _task_metrics_for(case) for case in cases if case.role is SnapshotRole.VULNERABLE
    )
    true_negative_results = tuple(
        _true_negative_for(case) for case in cases if case.role is SnapshotRole.PATCHED
    )

    mean_precision = (
        sum(m.precision for m in task_metrics) / len(task_metrics) if task_metrics else 0.0
    )
    mean_recall = sum(m.recall for m in task_metrics) / len(task_metrics) if task_metrics else 0.0
    file_f1 = sum(m.file_f1 for m in task_metrics) / len(task_metrics) if task_metrics else 0.0
    true_negative_rate = (
        sum(1 for r in true_negative_results if r.is_true_negative) / len(true_negative_results)
        if true_negative_results
        else 0.0
    )

    return CalibrationReport(
        task_metrics=task_metrics,
        true_negative_results=true_negative_results,
        mean_precision=mean_precision,
        mean_recall=mean_recall,
        file_f1=file_f1,
        true_negative_rate=true_negative_rate,
        vulnerable_case_count=len(task_metrics),
        patched_case_count=len(true_negative_results),
    )
