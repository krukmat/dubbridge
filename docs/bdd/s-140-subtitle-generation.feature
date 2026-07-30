---
type: BDD
title: "S-140 Subtitle Generation"
---
@S-140
Feature: Subtitle generation processing
  As the subtitle-generation pipeline,
  I want a ready transcription to produce a canonical subtitle artifact and
  enqueue human review through the existing review gate
  So that downstream review can consume real subtitle outputs without bypassing
  ADR-030's publication workflow.

  Scenario: S140_HP1 Ready transcription enqueues exactly one subtitle job
    Given an asset "video_01" has transcription status "READY"
    And the asset has more than one target language configured
    When the worker-runner prepares subtitle work for "video_01"
    Then the system should enqueue exactly one subtitle job for "video_01"
    And the job should target the deterministic first target-language route

  Scenario: S140_HP2 Successful subtitle processing persists subtitle output and enqueues review
    Given an asset "video_02" has a ready word-alignment artifact
    And subtitle segmentation succeeds for "video_02"
    When the worker-runner processes the subtitle job for "video_02"
    Then the system should persist one "subtitle" derived artifact for "video_02"
    And the subtitle status for "video_02" should become "READY"
    And the system should enqueue exactly one review task through the existing review-task path

  Scenario: S140_EC1 Missing alignment fails closed without creating review work
    Given an asset "video_03" has no word-alignment artifact
    When the worker-runner processes the subtitle job for "video_03"
    Then the subtitle status for "video_03" should become "FAILED"
    And no subtitle derived artifact should be persisted for "video_03"
    And no review task should be created for "video_03"

  Scenario: S140_EC2 Invalid segmentation output fails closed
    Given an asset "video_04" has a word-alignment artifact
    And subtitle segmentation returns invalid timing or empty output for "video_04"
    When the worker-runner processes the subtitle job for "video_04"
    Then the subtitle status for "video_04" should become "FAILED"
    And no subtitle derived artifact should be persisted for "video_04"
    And no review task should be created for "video_04"
