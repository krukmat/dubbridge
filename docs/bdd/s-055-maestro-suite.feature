Feature: Maestro screenshot and visual-audit suite
  As a DubBridge mobile maintainer
  I want a reproducible screenshot suite for the shipped mobile surfaces
  So that I can capture visual evidence without exposing raw backend credentials

  Scenario: SC-SUITE-1 Capture the unauthenticated auth surface
    Given the Android app launches from a clean state
    When the screenshot suite runs its unauthenticated phase
    Then it reaches the login screen and captures the auth-surface screenshot

  Scenario: SC-SUITE-2 Bootstrap an authenticated session without UI login
    Given the suite mints a seeded one-time mobile handoff code
    When the authenticated phase opens the bootstrap deep link
    Then the app redeems it into an opaque session and captures the authenticated home screen

  Scenario: SC-SUITE-3 Screenshot artifacts remain free of sensitive session values
    Given the suite has produced Maestro reports and screenshots
    When the runner sanitizes and verifies the output
    Then no handoff code or session reference remains in the persisted reports
