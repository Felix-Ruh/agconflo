// Component requirements that no test case verifies - a REVIEW QUEUE, not a
// gate, and the counterpart of unimplemented.cypher. A requirement is written
// before the case that checks it, so a gate here would refuse the commit that
// writes one: coverage is a query in this project, never a reason to refuse a
// commit.
//
// A row means nothing claims to check the requirement. That is expected while a
// feature is being specified, and worth reading afterwards - most of all for a
// requirement that already has code, since code with no test is the gap this
// queue exists to surface.
//
// Only `verifies` from a test_case counts. An implementation naming the
// requirement is not a test of it, and neither is a case that names it through
// ubc's built-in `links`; the fixture plants both.
MATCH (c:comp_req)
WHERE NOT (c)<-[:verifies]-(:test_case)
OPTIONAL MATCH (c)-[:allocated_to]->(k:comp)
RETURN c.id AS offender,
       k.title AS component,
       c.statement AS statement
ORDER BY offender
