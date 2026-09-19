// Test cases that no run executed - a REVIEW QUEUE, not a gate.
//
// A case gets a run when a test of that name is in the report the last test run
// wrote (DEC_LATEST_RUN_ONLY). So a row means one of three things, and only a
// person can tell them apart: the test has not been written yet, it was
// deleted, or it exists under a name that no longer matches its case. The third
// is the interesting one - a renamed test is caught by the check as a dead link
// from the run's side, but a case whose test quietly disappeared is visible
// only here.
//
// Not a gate, for the same reason as the other two queues: a test case is
// written before its test, so gating would refuse the commit that writes one.
//
// Only `executes` from a test_run counts. A run naming the case through ubc's
// built-in `links` is not an execution of it, and the fixture plants one.
MATCH (t:test_case)
WHERE NOT (t)<-[:executes]-(:test_run)
OPTIONAL MATCH (t)-[:verifies]->(r)
RETURN t.id AS offender,
       t.title AS title,
       r.id AS verifies
ORDER BY offender
