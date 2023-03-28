Feature: Capture logs
  Usually logging is considered as implementation details and is not be tested.
  But sometimes emitting certain logs can be considered as a feature why user wants.
  For example: System admin need access to error logs to be able to figure out problems.

  Scenario: Function should emit error log
    When Function is called
    Then Error log 'test1' is emitted

  Scenario: Function should not emit matching log
    When Function is called
    Then Error log 'test2' is not emitted
