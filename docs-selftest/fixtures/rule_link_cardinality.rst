==============================
Fixture: rule_link_cardinality
==============================

.. Both ends of "exactly one component": a requirement allocated to two, which
   means neither is answerable for it, and one allocated to none.

   The upper bound is a LOCAL maxItems while the lower bound is a NETWORK rule,
   and that split is forced rather than stylistic: measured, minItems does not
   fire when the link is absent altogether, because a constraint under
   `properties` only applies to a key that is present. Writing "exactly one" as
   minItems plus maxItems would therefore have silently accepted a requirement
   allocated to nothing.

   That is why the absent case is pinned HERE and not only in rule_link_targets,
   where the same rule is exercised by a link of the wrong type. The two halves
   fail through different mechanisms, so leaving one unpinned would let it decay
   quietly.

.. stkh_req:: A parent
   :id: STKH_CARDINALITY
   :stakeholder: user
   :statement: Agconflo shall hold one component answerable for each requirement.

.. feat_req:: A parent behaviour
   :id: FEAT_CARDINALITY
   :derived_from: STKH_CARDINALITY
   :ears_pattern: ubiquitous
   :verification_method: review
   :statement: Agconflo shall hold one component answerable for each requirement.

.. comp:: The first candidate
   :id: COMP_FIRST
   :crate: agconflo-core

.. comp:: The second candidate
   :id: COMP_SECOND
   :crate: agconflo-core

.. comp_req:: Allocated to two components at once
   :id: CREQ_TWO_COMPS
   :derived_from: FEAT_CARDINALITY
   :allocated_to: COMP_FIRST, COMP_SECOND
   :ears_pattern: ubiquitous
   :statement: The first candidate shall reject a divided allocation.

.. comp_req:: Allocated to nothing at all
   :id: CREQ_NO_ALLOC
   :derived_from: FEAT_CARDINALITY
   :ears_pattern: ubiquitous
   :statement: Some component shall be answerable for this requirement.
