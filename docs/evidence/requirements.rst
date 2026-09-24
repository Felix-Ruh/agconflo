===================================
Evidence about writing requirements
===================================

Measurements of the requirements themselves, as opposed to the tool that checks
them. Each is dated, and each says how to re-run it.

.. evd:: The first feature's requirements are all ubiquitous
   :id: EVD_EARS_FIRST_DISTRIBUTION
   :evd_kind: measurement
   :observed_on: 2026-09-18
   :observation: All twelve requirements written for the Context feature use the ubiquitous EARS pattern, and none uses the other five.

   Five feature requirements and seven component requirements. At the time they
   were the only needs carrying ``ears_pattern``, so the whole measurement was one
   query, grouping requirements that carry the field by pattern and level: it
   returned two rows, both ubiquitous.

   Re-running it later counts every feature written since, so the Context
   feature has to be selected explicitly: its feature requirements are the ones
   whose id begins ``FEAT_CONTEXT_``, and its component requirements are those
   derived from them.

.. evd:: Existing requirements were changed for other stakeholders' features
   :id: EVD_AMENDMENTS_CAME_SIDEWAYS
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: From #22 to #27, eight existing requirements changed what they oblige, six in their statements. Five of the six were changed by a feature under another stakeholder requirement, each having named the mechanisms of its day as the only ones.

   Measured with ``sh scripts/change-records.sh b7c894f`` on ``main`` at
   ``282acc2``, which names every requirement whose statement, pattern,
   verification method, stakeholder or placing links differ from those at #21:
   eight. Each statement change was then traced to its stakeholder requirement
   with ``sh scripts/impact.sh <ID>``, and set beside the stakeholder
   requirement of the feature that changed it.

   =======================================  ===  =======================  ======================
   Changed                                  In   Its own ancestor         Changed for
   =======================================  ===  =======================  ======================
   ``CREQ_RUN_REFUSES_HELD_IDENTIFIER``     #22  ``STKH_PROVENANCE``      ``STKH_PROVENANCE``
   ``CREQ_SOURCE_SOLE_ISSUER``              #25  ``STKH_PROVENANCE``      ``STKH_RESUMABLE_RUN``
   ``FEAT_BEHAVIOUR_FROM_SCRIPT``           #27  ``STKH_LIVE_BEHAVIOUR``  ``STKH_HUMAN_IN_RUN``
   ``CREQ_HOST_RUNS_THE_SCRIPT``            #27  ``STKH_LIVE_BEHAVIOUR``  ``STKH_HUMAN_IN_RUN``
   ``FEAT_BEHAVIOUR_REFUSED_BEFORE_START``  #27  ``STKH_WIRING_CHECKED``  ``STKH_HUMAN_IN_RUN``
   ``CREQ_BEHAVIOURS_REFUSE_MISSING``       #27  ``STKH_WIRING_CHECKED``  ``STKH_HUMAN_IN_RUN``
   =======================================  ===  =======================  ======================

   The five share one shape. Each statement closed the world on what existed
   when it was written: identifiers had "the only means" of being issued before
   a record could restore them, and every activation was performed "by running
   the script" - with a missing script the one way to have no behaviour - before
   a person could perform one. None of those closures was asked for by the
   parent. ``STKH_LIVE_BEHAVIOUR`` wants behaviour changeable without a rebuild,
   not every activation scripted; ``STKH_WIRING_CHECKED`` wants a node type
   nothing can perform refused before the run, not a scriptless one.

   The sixth is the other kind, and the kind the procedure allows: the identity
   slice found that requirement claiming less than its own parent, whose wording
   already said "holds", and brought it into line.

   The other two changes were links, both in #22: ``ARCH_RUN`` came to realise
   the new ``FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER``, and
   ``CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING`` gained it as a third parent. Both were
   within ``STKH_PROVENANCE``'s own line, and neither closed anything; they are
   counted because a placing link is part of what a requirement answers to.

   What the measurement does not show is that the five are wrong as they now
   stand. It shows how they came to be changed, which is the question the
   procedure answers; whether each is right against its own parents is the
   analysis a change request carries, and none has been carried yet.

.. evd:: The change-record check names exactly what history changed
   :id: EVD_CHANGE_CHECK_FIRES
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: Against the tree before #25 the change-record check named the five requirements changed since, and against the tree before #27 the four #27 changed. Against main itself it named none, and it refused 6 of 12 planted changes as designed.

   Run on ``main`` at ``282acc2``: ``sh scripts/change-records.sh 4c091f3``
   named ``CREQ_SOURCE_SOLE_ISSUER`` and the four of #27;
   ``sh scripts/change-records.sh 5dac701`` named only those four; and
   ``sh scripts/change-records.sh HEAD`` named none. The sets are the ones
   ``git log -p`` shows for the same ranges, so the check neither misses a
   change history holds nor invents one.

   ``sh scripts/change-records.sh --selftest`` plants twelve changes in a
   scratch repository and checks each in both modes, staged and by revision. It
   refuses a changed statement, pattern, ``derived_from`` or ``allocated_to``, a
   removed requirement, and a record naming the wrong id; it passes edited body
   prose, reordered links, a statement re-wrapped with the same words, an added
   requirement, a changed decision, and a changed statement whose record names
   it.
