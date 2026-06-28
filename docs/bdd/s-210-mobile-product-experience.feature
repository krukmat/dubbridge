Feature: Mobile product-experience refresh — dashboard, ergonomics, media-first
  As an authenticated DubBridge mobile user
  I want a dashboard home, bottom-anchored primary actions, media-led lists,
  human-readable detail screens, frictionless upload, and clear empty states
  So that the app feels intuitive, thumb-friendly, and media-first rather than
  a navigable database

  # ---------------------------------------------------------------------------
  # Home dashboard (D3)
  # ---------------------------------------------------------------------------

  Scenario: SC-DASH-1 Home dashboard shows live content on load
    Given I am an authenticated mobile user with pending review tasks and recent assets
    When I open the home screen
    Then I see a greeting that includes my identity
    And I see a pending-review summary with a count and at least one tappable task row
    And I see a recent-assets row with at least one asset entry
    And I see quick-action affordances for Assets, Upload, Review, and Organizations
    And I see an account entry that surfaces the sign-out affordance

  Scenario: SC-DASH-2 Home dashboard degrades cleanly on aggregate load error or session expiry
    Given I am an authenticated mobile user
    When the home dashboard aggregate fetch fails or the session expires
    Then the home screen shows a clear error or loading state without crashing
    And if the session expired I am redirected to the login screen

  Scenario: SC-DASH-3 Home quick-actions reach the correct sections
    Given I am on the home dashboard
    When I tap a quick-action affordance
    Then I navigate to the corresponding section screen
    And the home-screen testID and all home-open-* testIDs remain present and tappable

  # ---------------------------------------------------------------------------
  # Bottom-anchored primary actions (D2)
  # ---------------------------------------------------------------------------

  Scenario: SC-ACTBAR-1 Primary action is bottom-anchored on Upload, AssetDetail, and ReviewDetail
    Given I am viewing an Upload, AssetDetail, or ReviewDetail screen
    Then the primary action (Continue / Upload & finalize / Play / Approve / Reject)
      is rendered in a sticky action bar anchored at the bottom of the screen
    And the action bar respects the device safe-area bottom inset
    And the scrollable content area adds enough bottom padding that the last row
      is never occluded by the action bar

  # ---------------------------------------------------------------------------
  # Rights form: selectors + validation + step progress (D6)
  # ---------------------------------------------------------------------------

  Scenario: SC-FORM-1 Incomplete rights form shows a visible validation message
    Given I am on the upload rights form (step 1)
    When I tap Continue without filling all required fields
    Then a visible validation message identifies the missing field(s)
    And the form does not advance to step 2

  Scenario: SC-FORM-2 Rights form reflects a three-step progress indicator
    Given I am on the upload screen
    Then a step-progress indicator showing Rights → File → Finalize is visible
    And the indicator updates to reflect the current step as I advance through the flow

  # ---------------------------------------------------------------------------
  # Empty states with primary CTA (D7)
  # ---------------------------------------------------------------------------

  Scenario: SC-EMPTY-1 Empty list screens present a primary action
    Given I am an authenticated mobile user with no assets or no projects in a given list
    When I open an empty asset list or an empty project list
    Then I see a clear empty state
    And a primary CTA is visible that navigates me toward creating or uploading content

  # ---------------------------------------------------------------------------
  # User-facing status labels (D8)
  # ---------------------------------------------------------------------------

  Scenario: SC-STATUS-1 Domain status values render as user-facing labels
    Given I am viewing an asset, review task, or consent record with a known status
    Then the status is rendered as a user-facing label (e.g. "Ready" for finalized,
      "In review" for in_review)
    And the badge tone (color) matches the semantic status as before
    And the underlying domain status value is not shown raw to the user
