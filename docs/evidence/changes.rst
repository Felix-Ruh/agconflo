===============================================
Evidence about changes to existing requirements
===============================================

The impact analysis each change record in ``decisions/changes`` rests on, as it
was run: ``sh scripts/impact.sh <ID>`` for each requirement the record changes
or keeps, on the commit named. Re-running one later shows what has moved since.

.. evd:: The impact of a held identifier's refusal
   :id: EVD_IMPACT_RUN_REFUSES_HELD_IDENTIFIER
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: CREQ_RUN_REFUSES_HELD_IDENTIFIER answers to 2 needs, has 10 below it by link and 11 requirements sharing its component, and is named on 10 lines of prose.

   Run on ``71c060b``. Below it: ``IMPL_RUN_HOLDS``, ``IMPL_RUN_PRODUCED``, and
   four test cases with their runs -
   ``TEST_RUN_ACCEPTED_OUTPUTS_ARE_ALL_NEW``,
   ``TEST_RUN_OUTPUT_COMPOSING_ITS_INPUT_IS_ACCEPTED``,
   ``TEST_RUN_PASSED_THROUGH_ARGUMENT_IS_REFUSED`` and
   ``TEST_RUN_PASSED_THROUGH_OUTPUT_IS_REFUSED``. Beside it: every other
   requirement allocated to ``COMP_WORKFLOW_RUN``. No architecture realises a
   component requirement, so none is shared.

.. evd:: The impact of the identity feature's two placing links
   :id: EVD_IMPACT_ONE_CONTEXT_PER_IDENTIFIER_PLACED
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: ARCH_RUN answers to 17 needs and uses components holding 15 requirements. CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING answers to 5, has 7 below it and 11 beside it. Together they are named on 7 lines of prose.

   Run on ``71c060b``. ``ARCH_RUN`` realises ten features, behind which stand
   seven stakeholder requirements, and uses the workflow run and the run
   scheduler. The component requirement has ``IMPL_RUN_PRODUCED`` below it and
   three test cases with their runs:
   ``TEST_RUN_REFUSAL_DOES_NOT_SPEND_THE_BUDGET``,
   ``TEST_RUN_REFUSED_OUTPUT_CAN_BE_FAILED`` and
   ``TEST_RUN_REFUSED_OUTPUT_KEEPS_THE_ACTIVATION``.

   The first run of the analysis on ``ARCH_RUN`` reported nothing above or
   beside it, because the script followed only ``derived_from``, which an
   architecture does not have. ``71c060b`` fixed that before these counts were
   taken.

.. evd:: The impact of the identifier source being the only issuer
   :id: EVD_IMPACT_SOURCE_SOLE_ISSUER
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: CREQ_SOURCE_SOLE_ISSUER answers to 2 needs, has 6 below it by link and 1 requirement sharing its component, and is named on 7 lines of prose.

   Run on ``71c060b``. Below it: ``IMPL_ID_CONTEXT_ID`` and
   ``IMPL_RECORD_SOURCE``, and ``TEST_ID_CANNOT_BE_FORGED`` and
   ``TEST_RECORD_IDENTIFIERS_ONLY_WITH_A_SOURCE`` with their runs. Beside it:
   ``CREQ_SOURCE_NO_REPEAT``. In prose: two lines of ``id.rs`` and one of
   ``record.rs``, two of ``decisions/resume`` and two of
   ``evidence/requirements``.

.. evd:: The impact of performing a script's activations by running it
   :id: EVD_IMPACT_BEHAVIOUR_FROM_SCRIPT
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: FEAT_BEHAVIOUR_FROM_SCRIPT has 18 needs below it, 21 requirements beside it and 6 features sharing its architecture. CREQ_HOST_RUNS_THE_SCRIPT has 9 below it and 16 beside it. Each is named on 2 lines of prose.

   Run on ``71c060b``. Both answer to ``STKH_LIVE_BEHAVIOUR``, the component
   requirement through the feature. Below the feature: two component
   requirements, ``ARCH_BEHAVIOUR``, three code markers, and six test cases with
   their runs; the component requirement's nine are a subset of those.

.. evd:: The impact of refusing a run for a node type nothing performs
   :id: EVD_IMPACT_BEHAVIOUR_REFUSED_BEFORE_START
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: FEAT_BEHAVIOUR_REFUSED_BEFORE_START has 24 needs below it, 2 requirements beside it and 6 features sharing its architecture. CREQ_BEHAVIOURS_REFUSE_MISSING has 8 below it and 5 beside it. They are named on 2 and 6 lines of prose.

   Run on ``71c060b``. Below the feature: four component requirements
   (``CREQ_BEHAVIOURS_EVERY_FAULT``, ``CREQ_BEHAVIOURS_NOTHING_RUN``,
   ``CREQ_BEHAVIOURS_REFUSE_MISSING``, ``CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE``),
   ``ARCH_BEHAVIOUR``, three code markers, and eight test cases with their runs.
   Beside it: ``CREQ_BEHAVIOURS_PERSON_OR_SCRIPT`` and
   ``CREQ_BEHAVIOURS_REFUSE_TWICE``, which share the behaviour set without
   deriving from it.
