#!/usr/bin/env python3
"""Context capsule / attempt bundle schema (docs/plan/local-first-cloud-local-handoff.md).

Pure, offline validation for the shared structural envelope used at every
RRI handoff lane boundary. No network, no filesystem access beyond the dict
passed in by the caller.
"""

import hashlib
import json

CAPSULE_REQUIRED_FIELDS = (
    "work_item_id",
    "objective",
    "non_goals",
    "questions",
    "current_behavior",
    "required_behavior",
    "constraints",
    "allowed_paths",
    "acceptance_criteria",
    "repo_revision",
)

BUNDLE_REQUIRED_FIELDS = (
    "capsule_hash",
    "implementer_id",
    "model_tag",
    "start_ts",
    "end_ts",
    "diff_ref",
    "test_results",
    "review_verdict",
    "outcome",
)

VALID_OUTCOMES = ("success", "repair-needed", "escalated", "blocked")


class ValidationError(Exception):
    def __init__(self, field_name, reason):
        self.field_name = field_name
        self.reason = reason
        super().__init__(f"{field_name}: {reason}")


def _canonical_json(data):
    return json.dumps(data, sort_keys=True, separators=(",", ":"))


def compute_manifest_hash(capsule_fields):
    return hashlib.sha256(_canonical_json(capsule_fields).encode("utf-8")).hexdigest()


class Capsule:
    def __init__(self, fields, manifest_hash):
        self.fields = fields
        self.manifest_hash = manifest_hash

    def __getitem__(self, key):
        return self.fields[key]

    def get(self, key, default=None):
        return self.fields.get(key, default)


def validate_capsule(data):
    if not isinstance(data, dict):
        raise ValidationError("<root>", "capsule must be a dict")

    for field in CAPSULE_REQUIRED_FIELDS:
        if field not in data:
            raise ValidationError(field, "required field missing")

    fields = {name: data[name] for name in CAPSULE_REQUIRED_FIELDS}
    manifest_hash = compute_manifest_hash(fields)
    return Capsule(fields, manifest_hash)


class AttemptBundle:
    def __init__(self, fields):
        self.fields = fields

    def __getitem__(self, key):
        return self.fields[key]

    def get(self, key, default=None):
        return self.fields.get(key, default)


def validate_attempt_bundle(data, known_capsule_hashes):
    if not isinstance(data, dict):
        raise ValidationError("<root>", "attempt bundle must be a dict")

    for field in BUNDLE_REQUIRED_FIELDS:
        if field not in data:
            raise ValidationError(field, "required field missing")

    if data["outcome"] not in VALID_OUTCOMES:
        raise ValidationError(
            "outcome", f"must be one of {VALID_OUTCOMES}, got {data['outcome']!r}"
        )

    if data["capsule_hash"] not in known_capsule_hashes:
        raise ValidationError(
            "capsule_hash", f"unknown capsule hash {data['capsule_hash']!r}"
        )

    fields = {name: data[name] for name in BUNDLE_REQUIRED_FIELDS}
    return AttemptBundle(fields)
