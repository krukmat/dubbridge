Feature: Collaborative localization workspace
  As an authenticated DubBridge user
  I want to create and manage organizations, projects, and target languages
  So that my team can collaborate on localizing media content

  # ---------------------------------------------------------------------------
  # Organization management
  # ---------------------------------------------------------------------------

  Scenario: SC-ORG-1 Create an organization and become its owner
    Given I am an authenticated user with no organization
    When I create an organization
    Then I am its owner and can see it in my organization list

  # ---------------------------------------------------------------------------
  # Membership management
  # ---------------------------------------------------------------------------

  Scenario: SC-MEMBER-1 Invite a member with a role
    Given I am an org owner or admin
    When I add a member with the "reviewer" role
    Then that member can access the org with reviewer permissions

  Scenario: SC-MEMBER-2 Non-member is denied org access
    Given I am authenticated but not a member of an organization
    When I request that organization's projects
    Then I am denied access and no project data is returned

  # ---------------------------------------------------------------------------
  # Project management
  # ---------------------------------------------------------------------------

  Scenario: SC-PROJECT-1 Create a project and link assets
    Given I am an org owner or admin and own some assets
    When I create a project and link my assets to it
    Then the project lists those assets

  # ---------------------------------------------------------------------------
  # Target language intent
  # ---------------------------------------------------------------------------

  Scenario: SC-LANG-1 Declare target languages for a project
    Given I am viewing a project I can edit
    When I set a source language and one or more target languages
    Then the project records the localization intent
