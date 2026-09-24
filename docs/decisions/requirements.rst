====================================
Decisions about writing requirements
====================================

How requirements are written, as distinct from the tool that checks them. The
decisions filed beside the toolchain were forced by measured tool behaviour;
these are judgements about the material itself, made once there was material to
judge.

.. dec:: The six EARS patterns stand
   :id: DEC_EARS_SIX_PATTERNS
   :dec_status: accepted
   :decided_on: 2026-09-18
   :supported_by: EVD_EARS_FIRST_DISTRIBUTION
   :statement: Agconflo's requirements project shall keep the six EARS patterns as the only values of ears_pattern.

   The value set was provisional until real requirements existed to judge it
   against, and deliberately left alone until then. The first feature has now
   been written through every level, and nothing in it argues for a change: no
   statement was hard to classify, none fitted two patterns, and none needed a
   pattern that is missing. So the set is kept, including the absence of a
   separate value for quality attributes, for the reason ``ubproject.toml``
   gives beside the field.

   What the evidence does not show is that the other five are right. It shows
   they were not needed yet, and that is thinner than it looks: a value type has
   no triggers and no states, so a feature about one was never going to use
   ``event`` or ``state``. The prediction made before writing - mostly
   ubiquitous, a few event, at most one unwanted - was wrong for a reason worth
   knowing. The component requirements list their failure modes in their bodies
   rather than stating each as an ``unwanted`` requirement, so the one pattern
   written for failures had no occasion here. That convention will keep
   ``unwanted`` rare wherever it is followed.

   Kept rather than settled. The question reopens the first time a statement
   cannot be classified, and should be asked again once a feature with real
   triggers and states - the scheduler, a person taking part in a run - has been
   written. If a pattern is still unused then, that is evidence; now it is not.

.. dec:: A requirement changes only for its own parents
   :id: DEC_REQUIREMENT_ANSWERS_TO_ITS_PARENTS
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_AMENDMENTS_CAME_SIDEWAYS
   :statement: Agconflo's requirements project shall change an existing requirement only when its parents change or when it is found wrong against its own parents.

   This is what the V in the metamodel means, stated so that it cannot be
   worked around one feature at a time. A requirement is derived from its
   parents and verified by its own test cases; a change made for some other
   requirement's feature re-scopes it without its parents having asked, and
   everything below it - its derived requirements, its code, its tests - then
   answers to a need nobody traced.

   Measured, that is what had been happening (``EVD_AMENDMENTS_CAME_SIDEWAYS``):
   five of the first six statements ever changed were changed for a feature
   under another stakeholder requirement. Each had the same flaw, a statement
   naming the mechanisms of its day as the only ones, so the rule for writing a
   requirement and the rule for changing one are the same rule seen from two
   ends. A statement that claims no more than its parents need leaves nothing
   for an unrelated feature to collide with.

   Three alternatives were rejected.

   Amending as each feature needs was the practice until now, and it is the
   one measured above. Every such amendment was reviewed and argued, and every
   argument was the new feature's rather than the requirement's own parent's.

   Freezing requirements outright was the second. It would make an over-claim
   found against the requirement's own parent impossible to correct, which is a
   defect kept for the sake of a rule, and it gives a new feature no answer but
   to work around a requirement known to be wrong.

   A new identifier for every revision was the third. It keeps each version
   whole, and it breaks every link below the requirement on each revision,
   pushing the change onto requirements that did not change. A recorded change
   request gives the same history without that.

   When neither the existing requirement nor the new work can give way, two
   stakeholder requirements conflict, and that is resolved at the stakeholder
   level - by the stakeholder - before anything below it moves.

.. dec:: A change to a requirement is recorded with its impact analysis
   :id: DEC_REQUIREMENT_CHANGES_RECORDED
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_CHANGE_CHECK_FIRES
   :statement: Agconflo's requirements project shall record every change to an existing requirement as a decision supported by the impact analysis it rests on.

   A change request is carried out by a procedure (``AGENTS.md``, "The change
   procedure"), and the record is what makes the procedure auditable after the
   pull request is squashed: which requirement, the old and new statement, what
   raised it, which justification applied, and a verdict on every need the
   impact analysis found.

   It is a ``dec`` in ``decisions/changes.rst`` because a change to a
   requirement is a choice the project made and must not re-litigate, which is
   what a decision is. The impact analysis is an ``evd``, dated and
   re-runnable, because it is a measurement of the graph at that moment:
   ``scripts/impact.sh`` walks four directions - the parents, everything linked
   below, what shares a component or an architecture, and every mention in
   prose - and a later reader can run it again to see what has moved since.

   A need type of its own, with a link from the record to the requirement it
   amends, was the alternative. It would make "every change ever made to this
   requirement" one query instead of a text search, and it costs a new type, a
   new link, their rules, fixtures and golden files before a single record
   exists to show what the type needs to hold. Deferred rather than rejected:
   the text reference is the cheaper start, and the first time it fails to find
   a record is the evidence for the link.

   Commit bodies alone were the other, and lose twice: a squash merge folds them
   into one description, and nothing in the graph can reach them.

   What a machine can hold of this is held by ``scripts/change-records.sh``, in
   the commit hook and in CI: a changed statement, pattern, verification method,
   stakeholder or placing link on an existing requirement, or its removal, fails
   unless the same range adds a line to ``decisions/changes.rst`` naming it
   (``EVD_CHANGE_CHECK_FIRES``). That a record exists is checked; whether its
   analysis stands on the requirement's own parents is the review's.
