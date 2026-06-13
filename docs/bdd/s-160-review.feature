Feature: Human review and publication

  Scenario: SC-REVIEW-1 Reviewer sees their queue
    Given I am an org member with the reviewer role
    When I open the review queue
    Then I see the review tasks assigned to my org's projects

  Scenario: SC-REVIEW-2 Approve a derived output
    Given I am reviewing a pending review task
    When I approve it with a comment
    Then the task becomes approved and the decision is recorded immutably

  Scenario: SC-REVIEW-3 Reject a derived output
    Given I am reviewing a pending review task
    When I reject it with a comment
    Then the task becomes rejected and cannot be published

  Scenario: SC-PUBLISH-1 Publish a reviewed asset
    Given a review task is approved
    When I publish its asset and target
    Then a publication record is created and audited

  Scenario: SC-PUBLISH-2 Publication blocked without approval
    Given a review task is pending or rejected
    When I attempt to publish
    Then publication is refused with a clear review-required error

  Scenario: SC-NOTIFY-1 Reviewer notified of assignment
    Given a review task is assigned to me
    When the assignment happens
    Then I receive a notification referencing the task
