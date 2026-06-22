@S-125
Feature: HLS Playback Delivery
  As the playback-delivery boundary,
  I want to serve prepared HLS manifests and segments only through backend-owned,
  scoped, expiring grants so that prepared media is never exposed as raw object-store
  keys and is gated on readiness and authorization.

  Scenario: Authorized reviewer obtains a playback grant for a ready asset
    Given a prepared asset "video_01" with preparation status "READY"
    And an authenticated reviewer who is a member of the asset's org and project
    When the reviewer requests a playback grant for "video_01"
    Then a scoped, expiring playback grant should be issued
    And a durable grant audit row should be written
    And the response must not contain any raw object-store key

  Scenario: Grant issuance is denied for an asset that is not ready
    Given an asset "video_02" with preparation status "IN_PROGRESS"
    And an authenticated, authorized caller
    When the caller requests a playback grant for "video_02"
    Then grant issuance should be denied fail-closed
    And no playback grant should be created

  Scenario: Unauthorized caller cannot obtain a playback grant
    Given a prepared asset "video_03" with preparation status "READY"
    And a caller who is not a member of the asset's org or project
    When the caller requests a playback grant for "video_03"
    Then grant issuance should be denied before any grant is created
    And the refusal should be observable

  Scenario: Manifest is returned with short-lived scoped segment references only
    Given a valid playback grant for prepared asset "video_04"
    When the client fetches the manifest using the grant
    Then a rewritten ".m3u8" should be returned
    And every segment reference must be scoped and expiring
    And no raw object-store key must appear in the manifest

  Scenario: Segment fetched with an expired scoped reference is denied
    Given a client previously fetched the manifest for asset "video_05"
    And the scoped segment reference has since expired
    When the client fetches a segment using that scoped reference
    Then the segment request should be denied fail-closed
    And the previously fetched manifest must not grant durable access
