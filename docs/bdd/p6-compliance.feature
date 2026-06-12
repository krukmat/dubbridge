Feature: Compliance and consent center
  As an authenticated DubBridge asset owner
  I want to view governance events and manage voice-cloning consent
  So that my compliance obligations are visible and TTS derivatives are gated correctly

  # ---------------------------------------------------------------------------
  # Audit timeline
  # ---------------------------------------------------------------------------

  Scenario: SC-AUDIT-1 View an asset's audit timeline
    Given I own an asset with recorded governance events
    When I open its compliance view
    Then I see its audit events in chronological order

  Scenario: SC-AUDIT-2 Audit view is ownership-scoped
    Given an asset I do not own
    When I request its audit timeline
    Then I am denied and see no governance data

  # ---------------------------------------------------------------------------
  # Rights ledger
  # ---------------------------------------------------------------------------

  Scenario: SC-RIGHTS-1 View the rights ledger for an asset
    Given I own an asset with a rights record
    When I open its rights view
    Then I see its rights ledger entries

  # ---------------------------------------------------------------------------
  # Voice-consent ledger
  # ---------------------------------------------------------------------------

  Scenario: SC-CONSENT-1 Grant voice consent
    Given I own an asset
    When I grant voice-cloning consent with an evidence reference
    Then the consent is recorded as active

  Scenario: SC-CONSENT-2 Revoke voice consent
    Given an active voice consent exists for my asset
    When I revoke it
    Then the consent becomes inactive and the history is preserved

  Scenario: SC-CONSENT-3 Synthesis blocked without consent
    Given no active voice consent exists for an asset
    When a TTS or voice-cloning derivative is requested
    Then it is refused with a clear consent-required error
