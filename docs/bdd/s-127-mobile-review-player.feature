Feature: Mobile review player surface
  As an authenticated DubBridge mobile reviewer
  I want to play prepared HLS media inside the review and asset-detail surfaces
  So that I can validate media without leaving the mobile app.

  Scenario: SC-PLAYBACK-1 Review detail loads embedded playback after grant success
    Given I am viewing a review task whose asset can obtain a playback grant
    When the review detail screen opens
    Then a playback grant is issued for the task asset
    And the embedded player renders the manifest-backed media
    And the review decision controls remain available

  Scenario: SC-PLAYBACK-2 Review detail keeps the decision flow usable when playback is unavailable
    Given I am viewing a review task
    When playback grant issuance returns a denial or player-loading failure
    Then the screen shows a clear empty or error playback state
    And the review decision controls still remain available
    And no crash occurs

  Scenario: SC-PLAYBACK-3 Asset detail opens inline playback after an explicit play action
    Given I am viewing an asset detail whose asset status is "finalized"
    When I tap the Play action
    Then a playback grant is issued for that asset
    And playback opens inline in the same asset detail screen

  Scenario: SC-PLAYBACK-4 Asset detail denial or failure leaves the rest of the screen usable
    Given I am viewing an asset detail
    When playback grant issuance is denied or fails
    Then the inline playback area shows a clear empty or error state
    And the compliance entry and the rest of the asset metadata remain usable
