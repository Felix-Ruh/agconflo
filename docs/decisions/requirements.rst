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
