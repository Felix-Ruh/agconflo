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

.. evd:: The impact of what a model call sends
   :id: EVD_IMPACT_MODEL_WINDOW
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: FEAT_MODEL_WINDOW_IS_THE_PROMPT answers to 1 need and has 9 below it, 20 requirements beside it and 4 features sharing its architecture. CREQ_ROSTER_ONE_MESSAGE answers to 2 and has 3 below it and 4 beside it.

   Run on ``bae081b``. Below the feature: ``CREQ_HOST_PROMPT_IS_A_CONTEXT`` and
   ``CREQ_ROSTER_ONE_MESSAGE``, ``ARCH_MODELS``, ``IMPL_HOST_COMPLETE`` and
   ``IMPL_MODELS_CALL``, and ``TEST_HOST_PROMPT_MUST_BE_A_CONTEXT`` and
   ``TEST_MODELS_PROMPT_SENT_EXACTLY`` with their runs. Beside it: the four
   other requirements of the model roster and the sixteen of the script host.
   The component requirement's three below are ``IMPL_MODELS_CALL`` and
   ``TEST_MODELS_PROMPT_SENT_EXACTLY`` with its run. No line of prose names the
   feature, and the one naming the component requirement is that marker.

   ``CREQ_HOST_PROMPT_IS_A_CONTEXT``, which does not change, was run as well:
   2 above it, 3 below, 16 beside, and 2 lines of prose, both in ``host.rs``.

.. evd:: The impact of counting a run's budget
   :id: EVD_IMPACT_RUN_STOPS_AT_BUDGET
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: CREQ_RUN_STOPS_AT_BUDGET answers to 2 needs, has 9 below it by link and 11 requirements sharing its component, and is named on 3 lines of prose.

   Run on ``bae081b``. Below it: ``IMPL_RUN_STEP``, and four test cases with
   their runs - ``TEST_RUN_ACTIVATIONS_NEVER_EXCEED_BUDGET``,
   ``TEST_RUN_BUDGET_STOPS_AT_THE_LIMIT``,
   ``TEST_RUN_LAST_PERMITTED_ACTIVATION_COMPLETES`` and
   ``TEST_RUN_ZERO_BUDGET_ACTIVATES_NOTHING``. Beside it: every other
   requirement of ``COMP_WORKFLOW_RUN``. In prose: two lines of ``run.rs`` and
   one of ``tests/run``.

.. evd:: The impact of checking an output's declared type
   :id: EVD_IMPACT_RUN_REFUSES_UNDECLARED_OUTPUT
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: CREQ_RUN_REFUSES_UNDECLARED_OUTPUT answers to 2 needs, has 7 below it by link and 11 requirements sharing its component, and is named on 6 lines of prose.

   Run on ``bae081b``. Below it: ``IMPL_RUN_PRODUCED``, and three test cases
   with their runs - ``TEST_RUN_DESIGNATED_UNDECLARED_OUTPUT_IS_REFUSED``,
   ``TEST_RUN_OUTPUT_OF_DECLARED_TYPE_IS_ACCEPTED`` and
   ``TEST_RUN_UNDECLARED_OUTPUT_IS_REFUSED``. Beside it: every other
   requirement of ``COMP_WORKFLOW_RUN``. In prose: two lines of ``run.rs``, one
   of ``scheduler.rs``, one of ``features/behaviour`` and two of ``tests/run``.

.. evd:: The impact of stating the user a tool step runs as by what it may change
   :id: EVD_IMPACT_SANDBOX_STEP_USER
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: CREQ_SANDBOX_STEP_USER answers to 2 needs, has 6 below it by link and 9 requirements sharing its component, and is named on 2 lines of prose.

   Run as ``sh scripts/impact.sh CREQ_SANDBOX_STEP_USER`` on ``0e8fdb1``.
   Above it: ``FEAT_TOOL_KEPT_TO_ITS_GRANT`` and ``STKH_TOOLS_CONFINED``. Below
   it: ``IMPL_SANDBOX_STEP`` and ``IMPL_SANDBOX_STEP_USER``, and
   ``TEST_SANDBOX_STEP_USER`` and ``TEST_SANDBOX_USER_FROM_STATUS`` with their
   runs. Beside it: every other requirement of ``COMP_SANDBOX``. In prose: the
   two markers, both in ``sandbox.rs``. No architecture uses a component it
   is allocated to beyond its own.

.. evd:: The impact of refusing a run for its image and not its engine
   :id: EVD_IMPACT_RUNNER_REFUSES_UNGRANTED
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: CREQ_RUNNER_REFUSES_UNGRANTED answers to 3 needs, has 3 below it by link and 12 requirements sharing its component, and is named on 1 line of prose.

   Run as ``sh scripts/impact.sh CREQ_RUNNER_REFUSES_UNGRANTED`` on
   ``0e8fdb1``. Above it: ``FEAT_TOOL_UNGRANTED_REFUSED``,
   ``STKH_WIRING_CHECKED`` and ``STKH_TOOLS_CONFINED``. Below it:
   ``IMPL_RUNNER_PREPARED``, and ``TEST_RUNNER_REFUSES_UNGRANTED`` with its
   run. Beside it: every other requirement of ``COMP_RUNNER``. In prose: the
   marker, in ``runner.rs``.

.. evd:: The impact of stating what a tool container mounts by what the grants name
   :id: EVD_IMPACT_SANDBOX_LOCKED_DOWN
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: CREQ_SANDBOX_LOCKED_DOWN answers to 2 needs, has 7 below it by link and 15 requirements sharing its component, and is named on 5 lines of prose.

   Run as ``sh scripts/impact.sh CREQ_SANDBOX_LOCKED_DOWN`` on ``db9227d``.
   Above it: ``FEAT_TOOL_KEPT_TO_ITS_GRANT`` and ``STKH_TOOLS_CONFINED``. Below
   it: ``IMPL_SANDBOX_MAKE``, and ``TEST_SANDBOX_CONFINED_TO_GRANTS``,
   ``TEST_SANDBOX_OWN_PROCESS_SURVIVES`` and
   ``TEST_SANDBOX_ROOT_STEP_CLEANED_UP`` with their runs. Beside it: every
   other requirement of ``COMP_SANDBOX``, ``CREQ_SANDBOX_TRUSTS_GRANTED``
   among them. In prose: the marker in ``sandbox.rs``, two lines of
   ``components/environment``, one of ``decisions/changes`` and one of
   ``decisions/tools``. No architecture uses a component it is allocated to
   beyond its own.

.. evd:: The impact of removing the unbound-parameter defect
   :id: EVD_IMPACT_WIRING_REQUIRED_BOUND
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: FEAT_WIRING_REQUIRED_BOUND answers to 1 need, has 9 below it and 22 beside it, and is named on 1 line. CREQ_VALIDATOR_REQUIRED_BOUND answers to 2, has 7 below it and 12 beside it, and is named on 1 line.

   Run as ``sh scripts/impact.sh FEAT_WIRING_REQUIRED_BOUND`` and
   ``sh scripts/impact.sh CREQ_VALIDATOR_REQUIRED_BOUND`` on ``f862e04``.
   Above them: ``STKH_WIRING_CHECKED``. Below: ``CREQ_VALIDATOR_REQUIRED_BOUND``,
   ``ARCH_WIRING``, ``IMPL_WIRING_REQUIRED_BOUND``, and
   ``TEST_WIRING_DEGENERATE_DECLARATIONS_PASS``,
   ``TEST_WIRING_EVERY_UNBOUND_REQUIRED_IS_REPORTED`` and
   ``TEST_WIRING_UNWIRED_INSTANCE_IS_REPORTED`` with their runs. Beside them:
   every other requirement of ``COMP_WIRING_VALIDATOR``, and every other
   feature ``ARCH_WIRING`` realises. In prose: one line of ``features/run``
   and the marker in ``wiring.rs``.

.. evd:: The impact of stating a run's inputs as what nothing binds
   :id: EVD_IMPACT_RUN_ENTRY_SOURCE_EXACT
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: FEAT_RUN_ENTRY_SOURCE_EXACT answers to 1 need, has 13 below it and 29 beside it, and is named on no line. CREQ_RUN_REFUSES_UNFILLED_SIGNATURE answers to 2, has 11 below it and 20 beside it, and is named on 1 line.

   Run as ``sh scripts/impact.sh FEAT_RUN_ENTRY_SOURCE_EXACT`` and
   ``sh scripts/impact.sh CREQ_RUN_REFUSES_UNFILLED_SIGNATURE`` on ``f862e04``.
   Above them: ``STKH_EXPLICIT_CONTEXT``. Below: the component requirement,
   ``ARCH_RUN``, ``IMPL_RUN_SIGNATURE``, and
   ``TEST_RUN_ARGUMENT_FOR_NO_PARAMETER_IS_REFUSED``,
   ``TEST_RUN_ARGUMENT_OF_WRONG_TYPE_IS_REFUSED``,
   ``TEST_RUN_ENTRY_PARAMETER_ALSO_BOUND_IS_REFUSED``,
   ``TEST_RUN_MISSING_ARGUMENT_IS_REFUSED`` and
   ``TEST_RUN_SIGNATURE_FILLED_EXACTLY_STARTS`` with their runs. Beside them:
   every other requirement of ``COMP_WORKFLOW_RUN``, and every other feature
   ``ARCH_RUN`` realises. In prose: the marker in ``run.rs``.

.. evd:: The impact of giving text for any instance's parameter
   :id: EVD_IMPACT_RUNNER_ARGUMENTS
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: CREQ_RUNNER_ARGUMENTS_AS_TEXT and CREQ_RUNNER_REFUSES_UNKNOWN_ARGUMENT each answer to 2 needs, have 3 below them by link and 15 requirements sharing their component, and are named on 1 line of prose.

   Run as ``sh scripts/impact.sh CREQ_RUNNER_ARGUMENTS_AS_TEXT`` and
   ``sh scripts/impact.sh CREQ_RUNNER_REFUSES_UNKNOWN_ARGUMENT`` on
   ``42acfdc``. Above them: ``FEAT_RUNNER_STARTS_FROM_DOCUMENTS`` and
   ``STKH_RUN_FROM_DOCUMENTS``. Below: ``IMPL_RUNNER_ARGUMENTS``, and
   ``TEST_RUNNER_ARGUMENTS_KEPT_EXACTLY`` and
   ``TEST_RUNNER_UNKNOWN_ARGUMENT_REFUSED`` with their runs. Beside them:
   every other requirement of ``COMP_RUNNER``, each the other among them. In
   prose: the marker in ``runner.rs``.

.. evd:: The impact of stating quiescence as its parent does
   :id: EVD_IMPACT_SCHEDULER_NONE_READY
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: CREQ_SCHEDULER_NONE_READY answers to 2 needs, has 5 below it by link and 5 requirements sharing its component, and is named on 4 lines.

   Run as ``sh scripts/impact.sh CREQ_SCHEDULER_NONE_READY`` on ``041a406``.
   Above it: ``FEAT_RUN_QUIESCENCE_ENDS`` and ``STKH_STUCK_RUN``. Below it:
   ``IMPL_SCHEDULER_NONE_READY``, and
   ``TEST_SCHEDULER_NONE_READY_ONLY_WHEN_NONE`` and
   ``TEST_SCHEDULER_PARTIAL_INPUTS_STILL_QUIESCENT`` with their runs. Beside
   it: every other requirement of ``COMP_RUN_SCHEDULER``. In prose: the marker
   in ``scheduler.rs``, one line of ``components/run`` and two of
   ``tests/run``.

.. evd:: The impact of repetition's architecture realising one more feature
   :id: EVD_IMPACT_ARCH_REPETITION
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: ARCH_REPETITION answers to 3 needs, has nothing below it by link, uses components holding 35 requirements, and is named on 1 line.

   Run as ``sh scripts/impact.sh ARCH_REPETITION`` on ``041a406``. Above it:
   ``FEAT_REPEAT_ON_NEW_CONTEXTS``, ``FEAT_STANDING_SERVES_LATER_PASSES`` and
   ``STKH_REPETITION``. Beside it: every requirement of
   ``COMP_TOPOLOGY_READER``, ``COMP_RUN_SCHEDULER`` and ``COMP_WORKFLOW_RUN``.
   In prose: one line of ``components/repetition``.
