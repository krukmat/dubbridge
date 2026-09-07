@S-120
Feature: Media Preparation Processing
  As a media processing system,
  I want to ensure that assets are correctly prepared, metadata is extracted, and HLS segments are generated before downstream consumption.

  Scenario: S120_HP1 Successful preparation produces metadata and HLS outputs
    Given an uploaded asset "video_01"
    When the preparation pipeline processes "video_01"
    Then the system should generate valid metadata for "video_01"
    And the system should produce a complete set of HLS segments
    And the asset preparation status should be marked as "READY"

  Scenario: S120_EC1 Downstream processing is blocked while asset is not prepared
    Given an uploaded asset "video_02"
    And the preparation pipeline has not finished for "video_02"
    When a downstream consumer requests "video_02"
    Then the request should be rejected or deferred with a "NOT_READY" outcome

  Scenario: S120_EC2 Preparation failure leaves the asset not ready and observable
    Given an uploaded asset "video_03"
    When the preparation pipeline fails for "video_03" due to a transcoding error
    Then the system should record observable error detail
    And the asset preparation status should remain "FAILED" or otherwise not ready
    And downstream consumers must not treat the asset as prepared

  Scenario: S120_EC3 Malformed probe or transcode result does not mark the asset prepared
    Given an uploaded asset "video_04"
    When the probe or transcode returns malformed output for "video_04"
    Then the system must not mark "video_04" as "READY"
    And the preparation failure must remain observable
