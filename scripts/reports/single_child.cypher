// Requirements that are the only child of their parent - a REVIEW QUEUE, not a
// gate. A parent with exactly one child is where restatement hides, since a
// child that says nothing new has no sibling to be compared against. It is also
// often legitimate: a goal can refine into exactly one behaviour, and a
// behaviour can be allocated to exactly one component. So each row is read by a
// person, and nothing fails on it.
//
// `same_response` is the one mechanical signal, and the strongest: the part of
// the statement after " shall " is identical in child and parent, so only the
// subject changed - or nothing did. That is restatement the subject-agreement
// gate cannot see, because the subject is exactly what differs. Rows where it is
// true come first. Nothing subtler is attempted: this Cypher has no measure of
// textual similarity, and a threshold nobody can justify is worse than none.
//
// Covers every level joined by derived_from, which is where a requirement
// refines another; feat_arch and comp are not requirements and are left out.
MATCH (c)-[:derived_from]->(p)
WITH p, count(c) AS n
WHERE n = 1
MATCH (c)-[:derived_from]->(p)
RETURN c.id AS offender,
       p.id AS parent,
       split(c.statement, " shall ")[1] = split(p.statement, " shall ")[1] AS same_response,
       p.statement AS parent_statement,
       c.statement AS child_statement
ORDER BY same_response DESC, offender
