============================
A person in a run test cases
============================

How each requirement in ``components/person`` is to be checked, with two
feature-level cases where the claim is about a whole run. Results are never
written here: they are imported from the test runner.

A case's id is the path of the Rust test that implements it, uppercased. The
behaviour set's cases live in ``agconflo-lua``'s module ``behaviours`` and the
script host's in its module ``scripted``, which drives a run.

No person takes part in a test. Where one would answer, the test supplies text
in their place, which is all a person's answer is to the run - a context
arriving from outside it - and a new process is stood in for as the resume
cases do: a run built from the record's text and nothing else. A step answered
in a real second process, against a real model, was measured once outside the
tests (``EVD_PERSON_ANSWERED_IN_A_NEW_PROCESS``).

Every failure mode listed in ``components/person`` is named by the case that
catches it, and one case checks ``CREQ_BEHAVIOURS_REFUSE_MISSING`` for a
behaviour supplied by naming a person rather than by a script.

.. test_case:: A node type a person performs needs no script
   :id: TEST_BEHAVIOURS_PERSON_NEEDS_NO_SCRIPT
   :verifies: CREQ_BEHAVIOURS_REFUSE_MISSING
   :test_kind: positive
   :coverage: partial

   A workflow whose two node types are given a script and named as performed by
   a person respectively starts, and is not refused for the second type's
   missing script. Without the naming, the same behaviours are refused for it.

.. test_case:: A node type given a script and a person is refused
   :id: TEST_BEHAVIOURS_PERSON_AND_SCRIPT_REFUSED
   :verifies: CREQ_BEHAVIOURS_PERSON_OR_SCRIPT
   :test_kind: error_path
   :coverage: full

   A node type named as performed by a person and given two scripts is refused
   once, naming the type and both documents, and alongside a missing script for
   another type, so both faults arrive in one refusal. Nothing is run.

   Controls that must start: a type named twice as performed by a person, and a
   type no instance names that is both named and given a script.

   Catches: the script run; the person asked; refused for a type no instance
   names.

.. test_case:: A person's step is handed to the caller, with nothing run for it
   :id: TEST_SCRIPTED_PERSON_STEP_HANDED_OVER
   :verifies: CREQ_HOST_HANDS_OVER_PERSON_STEP
   :test_kind: positive
   :coverage: full

   A chain of a scripted step, a person's step and a scripted step whose script
   raises an error if it runs. The run returns the person's activation, for the
   second instance, carrying as its input the very context the first produced
   under the identifier the last record holds for it, and asking for the type
   the second's node type declares. It returns neither an ending nor a failure,
   the third script has not run, and two records were handed over: at the start
   and after the first output.

   Catches: an activation other than the one offered; a failure reported for
   it; the run asked for its next step and told it ended; a panic.

.. test_case:: A person's text is the output of the step it was supplied for
   :id: TEST_SCRIPTED_PERSON_TEXT_BECOMES_OUTPUT
   :verifies: CREQ_HOST_TAKES_PERSON_TEXT
   :test_kind: property
   :coverage: partial

   For any text - empty, whitespace at either end, line endings of both kinds,
   characters outside ASCII - the chain above with a passing third script,
   answered from its last record with that text, completes with a budget exactly
   its length. The person's step produced a context of its declared type
   rendering exactly the text, a record was handed over after the answer and
   after the third output, and the record after the answer resumes.

   Catches: a context from a fresh source; the text changed on the way; a
   record not handed over after it.

.. test_case:: A person's answer to a record with an exhausted source fails the step
   :id: TEST_SCRIPTED_EXHAUSTED_SOURCE_FAILS_THE_STEP
   :verifies: CREQ_HOST_TAKES_PERSON_TEXT
   :test_kind: error_path
   :coverage: partial

   The chain above parked at the person's step, its record's source rewritten
   as exhausted - a sound record, since such a source has issued every
   identifier it holds - and answered. The run ends with the person's step
   failed, carrying that the source was exhausted.

   Catches: an exhausted source met with a panic, or the answer dropped.

.. test_case:: An answer for a step the run does not await is refused, with nothing run
   :id: TEST_SCRIPTED_ANSWER_ELSEWHERE_REFUSED
   :verifies: CREQ_HOST_REFUSES_ANSWER_ELSEWHERE
   :test_kind: error_path
   :coverage: full

   Each script in these runs raises an error if it runs, so running one ends
   the run on a failure rather than refusing. A record whose run next offers a
   script's step is refused, answered for the person's instance and for the
   script's own, naming the script's; a record awaiting the person, answered
   for another instance or for one the workflow does not have, is refused
   naming the person's; and a record of a run that has completed is refused
   naming none. No record is handed over for any of them.

   A record that is not a record of the workflow is refused as a resume
   refuses it, and behaviours with a fault as their check refuses them, whether
   or not the answer names the step the run awaits.

   Catches: taken for a script's step; scripts run until the instance comes
   up; dropped for a run that has ended; refused without naming what the run
   awaits.

.. test_case:: A run awaiting a person is not reported stuck
   :id: TEST_SCRIPTED_AWAITING_IS_NOT_QUIESCENT
   :verifies: FEAT_PERSON_WAIT_NOT_STUCK
   :test_kind: positive
   :coverage: full

   A workflow whose last step is a person's, with nothing else left to run,
   returns that step rather than ending quiescent, and resumed from its record
   returns the same step again. The control is a person's step that no run can
   reach, in a cycle written in bindings: that run ends quiescent, naming it.

.. test_case:: A person between two model calls repeats neither
   :id: TEST_SCRIPTED_PERSON_BETWEEN_MODEL_CALLS
   :verifies: FEAT_PERSON_TEXT_IS_THE_OUTPUT
   :test_kind: positive
   :coverage: full

   Three steps: a model call, a person, a model call. The run returns the
   person's step having made one call. Answered from its last record against a
   provider that answers, it makes one call, for the third step, and completes
   with the first answer, the person's text and the second answer in its
   result, whose lineage holds the identifiers the first half recorded.
