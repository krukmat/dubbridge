Feature: Mobile asset lifecycle
  As an authenticated DubBridge mobile user
  I want to browse, open, and create assets
  So that I can manage my media content from the mobile app

  # ---------------------------------------------------------------------------
  # List surface
  # ---------------------------------------------------------------------------

  Scenario: SC-LIST-1 Browse my assets
    Given I am an authenticated mobile user with at least one owned asset
    When I open the asset list
    Then I see each of my assets with its title and status

  Scenario: SC-LIST-2 Empty asset list
    Given I am an authenticated mobile user with no owned assets
    When I open the asset list
    Then I see a clear empty state and no error

  # ---------------------------------------------------------------------------
  # Detail surface
  # ---------------------------------------------------------------------------

  Scenario: SC-DETAIL-1 Open an asset from the list
    Given I am viewing my populated asset list
    When I tap an asset
    Then I see its detail with title, status, asset id, and uploader id

  # ---------------------------------------------------------------------------
  # Ingestion flow
  # ---------------------------------------------------------------------------

  Scenario: SC-INGEST-1 Upload a new asset
    Given I am an authenticated mobile user
    When I pick a file, submit valid rights, and finalize
    Then the asset is created and appears in my asset list

  Scenario: SC-INGEST-2 Upload rejected without rights
    Given I have uploaded a file but not submitted rights
    When I attempt to finalize
    Then finalization is rejected and I see a clear rights-required error
