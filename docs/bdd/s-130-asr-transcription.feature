---
type: BDD
title: "S-130 ASR Transcription"
---
@S-130
Feature: ASR transcription processing
  As the ASR processing pipeline,
  I want a prepared asset to produce transcript and alignment artifacts through the
  worker boundary
  So that downstream subtitle generation can consume a ready transcription state.

  Scenario: S130_HP1 Prepared asset produces transcript and alignment artifacts
    Given an asset "video_01" has preparation status "READY"
    And the transcription worker can read the prepared source audio for "video_01"
    When the worker-runner processes the transcription job for "video_01"
    Then the system should persist one "transcript_text" derived artifact for "video_01"
    And the system should persist one "word_alignment" derived artifact for "video_01"
    And the transcription status for "video_01" should become "READY"

  Scenario: S130_EC1 ASR worker failure marks the asset transcription as failed
    Given an asset "video_02" has preparation status "READY"
    And the transcription worker returns an ASR failure for "video_02"
    When the worker-runner processes the transcription job for "video_02"
    Then the transcription status for "video_02" should become "FAILED"
    And the failure should include observable error detail
    And no transcript or alignment derived artifact should be persisted for "video_02"
