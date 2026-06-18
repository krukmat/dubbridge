Feature: Mobile credential login with backend-issued JWT (FenixCRM parity)

  # Governed by ADR-031. Slice S-200. Source reference:
  # /Users/matias/fenix/docs/mobile-auth-flow-reference.md

  Scenario: SC-AUTH-1 Sign in with valid credentials
    Given I am an unauthenticated user with a registered account
    When I submit my email and password
    Then the backend issues a token and I land on the home surface

  Scenario: SC-AUTH-2 Register a new account and workspace
    Given I am a new user with no account
    When I register with an email, a password of at least 12 characters, and a workspace name
    Then my workspace and account are created atomically and I am signed in

  Scenario: SC-AUTH-3 Stored token restores the session on cold start
    Given I have previously signed in on this device
    When I cold-start the app
    Then I land on the home surface without seeing the login screen

  Scenario: SC-AUTH-4 Invalid credentials are rejected generically
    Given I am on the login screen
    When I submit an unknown email or a wrong password
    Then I see the same generic invalid-credentials error and nothing is stored

  Scenario: SC-AUTH-5 Expired or rejected token forces logout
    Given I am signed in
    When an authenticated request returns 401
    Then my stored token is cleared and I am returned to the login screen

  Scenario: SC-AUTH-6 Logout clears the stored token
    Given I am signed in
    When I log out
    Then the token is removed from secure storage and I am unauthenticated

  Scenario: SC-AUTH-7 Registration rejects a duplicate email
    Given an account already exists for an email
    When I register again with the same email
    Then registration is refused as a conflict and no second account is created

  Scenario: SC-AUTH-8 Algorithm substitution is rejected at parse time
    Given a token whose header algorithm is not HS256
    When the backend verifies it
    Then verification fails closed before any signature or claim check
