// Component requirements that no code answers for - a REVIEW QUEUE, not a gate.
// A component requirement is written before its code, and a feature's first
// commits have no code at all, so a gate here would refuse exactly that work: a
// coverage rule by another name, which this project does not have (AGENTS.md).
// So each row is read by whoever reviews the change, and nothing fails on it.
//
// A row means no marker in the Rust source names the requirement in its
// `implements` field. The ordinary reason is that the code is not written yet.
// The other is a marker that is missing or names the wrong requirement. One
// naming a requirement that does not exist is a dead link, which fails
// `ubc check` by itself, and the requirement it should have named still
// appears here - so a broken marker shows up as a row rather than as nothing.
//
// Only `implements` from an `impl` counts. A test case verifying the
// requirement is not code; nor is another type naming it through `implements`,
// which any need may carry since links are not declared per type; nor is an
// implementation naming it through ubc's built-in `links`. The fixture plants
// all three.
//
// The component is optional in the match so that a requirement missing its
// allocation, which a schema rule refuses anyway, still appears rather than
// dropping out of the queue.
MATCH (c:comp_req)
WHERE NOT (c)<-[:implements]-(:impl)
OPTIONAL MATCH (c)-[:allocated_to]->(k:comp)
RETURN c.id AS offender,
       k.title AS component,
       k.crate AS crate,
       c.statement AS statement
ORDER BY offender
