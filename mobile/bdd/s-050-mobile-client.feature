Feature: First-party mobile client
  As an authenticated DubBridge mobile user
  I want to sign in through the session gateway and browse available asset surfaces
  So that I can use the first-party mobile app without handling raw backend tokens

  Scenario: SC-AUTH-1 Sign in through the mobile gateway handoff
    Given I launch the mobile app with valid runtime configuration
    When I complete the system-browser login and the gateway returns a valid handoff code
    Then the app stores an opaque session reference and shows the authenticated home

  Scenario: SC-AUTH-2 Login fails closed when the handoff is missing or invalid
    Given I start the mobile login flow
    When the browser callback is cancelled, missing its handoff code, or the gateway rejects the redemption
    Then I remain unauthenticated and see a clear login failure state

  Scenario: SC-AUTH-3 Token-like session values are rejected on device
    Given the device receives a token-like session value from storage, rotation, or login redemption
    When the app evaluates that value
    Then it is rejected and not persisted as the mobile session reference

  Scenario: SC-NAV-1 Auth state controls the root navigation tree
    Given the app resolves its auth state and runtime configuration
    When the user is authenticated, unauthenticated, loading, or misconfigured
    Then the app shows the correct home, login, or configuration-error surface

  Scenario: SC-ASSET-1 Browse my asset list and open asset detail
    Given I am an authenticated mobile user with available assets
    When I open the asset list and select one asset
    Then I see the asset detail with its available summary and status

  Scenario: SC-ASSET-2 Asset surfaces handle empty, failed, or unavailable responses clearly
    Given I am an authenticated mobile user
    When the asset list is empty, the gateway request fails, or the mobile surface is not yet available
    Then I see a clear empty, error, or not-available state without the app crashing
