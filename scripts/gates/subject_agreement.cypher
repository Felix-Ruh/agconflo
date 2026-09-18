// A component requirement's grammatical subject must be the exact title of the
// component it is allocated to.
//
// Why: the subject is what makes a requirement that merely restates its parent
// visible. "Agconflo shall bind only wired contexts" copied down a level passes
// every schema rule, and allocates nothing.
//
// The subject is found by position, not by prefix. Across all six EARS
// grammars in docs/schemas.json it sits at the start of the statement, after
// ", ", or after ", then ", and it ends at " shall ". The trigger and state
// clauses cannot contain a comma and the one-obligation rule forbids a second
// shall, so the only " shall " in a statement is the subject's. This gate leans
// on those rules, and would need revisiting if they were loosened.
//
// Measured against docs-selftest/fixtures/gate_subject_agreement.rst: the first
// version of this query tested `STARTS WITH b.title`, and it reported five valid
// requirements there - every EARS pattern but ubiquitous opens with a clause -
// while missing a statement whose subject merely began with the title.
MATCH (a:comp_req)-[:allocated_to]->(b:comp)
WHERE NOT (a.statement STARTS WITH b.title + " shall "
        OR a.statement CONTAINS ", " + b.title + " shall "
        OR a.statement CONTAINS ", then " + b.title + " shall ")
RETURN a.id AS offender, b.title AS component
