=====================
Fixture: report_unrun
=====================

.. The fixture for the unrun-cases review report in scripts/reports/. Every need
   here is valid by every schema rule, so the golden records nothing: a test case
   is written before the test that runs it, and no rule may refuse it for that.

   Every need whose id contains _BAD_ must appear in the report, and no other
   may. One offender is named by a run through ubc's built-in links rather than
   executes, which a looser query would take for an execution. The controls are
   a case whose run passed, a case whose run failed - an outcome is still a run -
   and the requirement the cases verify, which has no run of its own and must
   not appear: only a test case is in this queue.

.. stkh_req:: A node sees only what was wired to it
   :id: STKH_UNRUN
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   Not a test case. Must not appear.

.. feat_req:: Binding a node's parameters
   :id: FEAT_UNRUN
   :derived_from: STKH_UNRUN
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall bind only the contexts wired to a node's parameters.

   Verified by the cases below, and never executed itself. Must not appear.

.. test_case:: Its test passed
   :id: TEST_OK_PASSED
   :verifies: FEAT_UNRUN
   :test_kind: positive
   :coverage: partial

   Must not appear.

.. test_case:: Its test failed
   :id: TEST_OK_FAILED
   :verifies: FEAT_UNRUN
   :test_kind: property
   :coverage: partial

   A failing test is still a test that ran, and the queue asks whether a case
   ran at all. Must not appear.

.. test_case:: Never executed
   :id: TEST_BAD_NO_RUN
   :verifies: FEAT_UNRUN
   :test_kind: error_path
   :coverage: partial

   Either its test is unwritten, or it was deleted, or it now has a name that
   no longer matches this case. Must appear.

.. test_case:: Named by a run through links rather than executes
   :id: TEST_BAD_ONLY_LINKED
   :verifies: FEAT_UNRUN
   :test_kind: positive
   :coverage: partial

   A run mentions it without claiming to have executed it. Must appear.

.. test_run:: The passing run
   :id: RUN_OK_PASSED
   :executes: TEST_OK_PASSED
   :test_outcome: passed

   The only link into the case it executed.

.. test_run:: The failing run
   :id: RUN_OK_FAILED
   :executes: TEST_OK_FAILED
   :links: TEST_BAD_ONLY_LINKED
   :test_outcome: failed

   Executes one case and merely mentions another, which is what makes the link
   type load-bearing here.
