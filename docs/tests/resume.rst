=========================
Resuming a run test cases
=========================

How each requirement in ``components/resume`` is to be checked, with one
feature-level case where the claim is about a whole run. Results are never
written here: they are imported from the test runner.

A case's id is the path of the Rust test that implements it, uppercased. The
run record's cases live in the core's module ``record`` and the scripted run's
in ``agconflo-lua``'s module ``scripted``, neither of which the other crate has.

An interruption is a scripted run dropped while it waits on a model call,
which is how the measurement behind this feature was taken
(``EVD_INTERRUPTED_RUN_REPEATS_CALLS``). It ends the run the way a process
ending does, short of the process: nothing after the drop runs. What a real
restart adds - the memory gone - cannot leak into a resumed run here either,
because a resumed run is built from the record's text and nothing else. A real
restart was measured once, outside the tests, against a real model
(``EVD_RESUMED_ACROSS_A_RESTART``).

Every failure mode listed in ``components/resume`` is named by the case that
catches it, and one case verifies the requirement of ``components/context``
that the record amended.

.. test_case:: A record holds the run, flat and once each
   :id: TEST_RECORD_HOLDS_THE_RUN
   :verifies: CREQ_RECORD_HOLDS_THE_RUN
   :test_kind: positive
   :coverage: partial

   A small run recorded with an activation outstanding, read back as TOML
   rather than resumed. It holds the budget, the activations spent, the
   argument, each accepted output in order with its inputs by identifier, and
   each context once, under its identifier, with any parts named rather than
   written inside it - including a part two contexts hold.

   Catches: the activations spent left out; the inputs left out; a composition
   written inside its parent; a context written once per holder.

.. test_case:: A resumed run ends as the uninterrupted run did
   :id: TEST_RECORD_RESUMED_RUN_ENDS_ALIKE
   :verifies: CREQ_RECORD_CONTINUES_THE_RUN
   :test_kind: property
   :coverage: partial

   For any well-formed workflow and any budget, a run driven to its ending, and
   the same run recorded at each point along the way - between activations and
   with one outstanding - and resumed. Each resumed run offers the activations
   the uninterrupted one offered after that point, in the same order and with
   the same inputs, and ends the same way having spent the same activations.

   Catches: the outstanding activation lost; the recorded outputs handed to the
   caller to perform again.

.. test_case:: An outstanding activation is not charged again
   :id: TEST_RECORD_OUTSTANDING_NOT_CHARGED_AGAIN
   :verifies: CREQ_RECORD_CONTINUES_THE_RUN
   :test_kind: error_path
   :coverage: partial

   A chain with a budget exactly its length, recorded while its last
   activation is outstanding and resumed. The resumed run offers that
   activation, is given its output and completes, rather than ending on its
   budget.

   Catches: the outstanding activation charged again.

.. test_case:: A resumed context is the context recorded
   :id: TEST_RECORD_CONTEXTS_KEPT
   :verifies: CREQ_RECORD_KEEPS_CONTEXTS
   :test_kind: property
   :coverage: partial

   For any text - whitespace at either end, line endings of both kinds, NUL and
   other control characters, characters outside ASCII - and for compositions
   of such texts, some sharing parts and one of no parts at all, a run holding
   them recorded and resumed. Each resumed context has the identifier, type,
   rendering and parts it was recorded with, and a part two compositions hold
   is one value in the resumed run.

   Catches: text changed on the way; a part made once per holder.

.. test_case:: A deep composition is resumed without recursion
   :id: TEST_RECORD_DEEP_COMPOSITION_KEPT
   :verifies: CREQ_RECORD_KEEPS_CONTEXTS
   :test_kind: error_path
   :coverage: partial

   A run whose argument is a composition 100,000 levels deep, recorded and
   resumed on a thread with a small stack. It resumes, and the argument renders
   as it did.

   Each level comes from a source of its own standing below its part's, so that
   identifiers fall outward: the outermost is written first, and making it goes
   down every level. Numbered the usual way, each part would already have been
   made when its holder was reached, and a recursive maker would never go deeper
   than one - measured, it passed this case that way.

   Catches: a deep composition built by recursion.

.. test_case:: A resumed run's source issues nothing the recorded one had
   :id: TEST_RECORD_SOURCE_CONTINUES
   :verifies: CREQ_RECORD_SOURCE_CONTINUES
   :test_kind: property
   :coverage: full

   For any number of contexts made and dropped before a run is recorded, the
   source that comes back with the resumed run issues none of the identifiers
   the recorded source issued, dropped ones included. An exhausted source comes
   back exhausted.

   Catches: a fresh source; a source continuing from the highest identifier
   held; an exhausted source resumed as fresh.

.. test_case:: A record the workflow would not have produced is refused
   :id: TEST_RECORD_DIVERGED_RECORD_REFUSED
   :verifies: CREQ_RECORD_REFUSES_DIVERGENCE
   :test_kind: error_path
   :coverage: full

   The measured shape and its neighbours, each resumed against a workflow that
   would not have produced it: an instance bound to another input; the same
   instances in another order; an output recorded for an instance the workflow
   does not have; an output after the designated one; an output of a type the
   workflow no longer declares; and activations spent below the outputs and two
   above them. Each is refused naming the first recorded output that differs,
   or the count.

   Catches: only instance names compared; recorded outputs left over; an output
   the run refuses reported anyway; more activations spent than a run could have
   spent.

.. test_case:: A record whose run would not start is refused as the start refuses
   :id: TEST_RECORD_START_REFUSAL_CARRIED
   :verifies: CREQ_RECORD_REFUSES_WHAT_START_REFUSES
   :test_kind: error_path
   :coverage: full

   A sound record resumed against a workflow with a wiring defect, and against
   one whose entry parameter the recorded argument does not fill. Each is
   refused carrying the refusal a start with the same arguments gives.

   Catches: the refusal reported as a divergence; the start's checks skipped.

.. test_case:: A text that is not a record is refused with its fault
   :id: TEST_RECORD_UNREADABLE_REFUSED
   :verifies: CREQ_RECORD_REFUSES_UNREADABLE
   :test_kind: error_path
   :coverage: full

   A sound record, each time damaged in one way: not TOML at all; another
   version; a field missing; a field added; a value of the wrong kind; an
   identifier written as ``07``; a part naming no recorded context; a
   composition holding itself; an identifier at the source's position; a
   context nothing holds. Each is refused with its own fault, at a line and
   column, and under the key for all but the first; the undamaged record
   resumes.

   Catches: any TOML accepted; an unknown field ignored; an identifier written
   in another form; a part naming no recorded context, or a composition holding
   itself; an identifier at or past the recorded source's position; a context
   nothing holds, kept.

.. test_case:: A record's identifiers come back only beside a source past them
   :id: TEST_RECORD_IDENTIFIERS_ONLY_WITH_A_SOURCE
   :verifies: CREQ_RECORD_SOURCE_CONTINUES
   :test_kind: error_path
   :coverage: partial

   A record holding an identifier one past its source's position, and one at
   the highest a source can issue with the source recorded as exhausted. The
   first is refused. The second resumes, and its source issues nothing.

   Catches: a record's identifiers resumed without a source past them.

.. test_case:: A scripted run hands over a record at the start and after each output
   :id: TEST_SCRIPTED_RECORDS_HANDED_OVER
   :verifies: CREQ_HOST_HANDS_RECORDS
   :test_kind: positive
   :coverage: full

   A chain of three scripted nodes run to completion, and one whose second node
   fails. The first hands over four records, the second two; the record handed
   over after each accepted output holds that output and no later one, and
   each resumes against the workflow - which a record claiming a position of
   zero would not, since every identifier it holds would be past it.

   Catches: handed only at the end; handed before the output is accepted; the
   position read from the caller's source while it is lent.

.. test_case:: An interrupted scripted run resumes without repeating a call
   :id: TEST_SCRIPTED_INTERRUPTED_RUN_RESUMES
   :verifies: FEAT_RESUME_REPEATS_NO_OUTPUT
   :test_kind: positive
   :coverage: full

   The measured shape (``EVD_INTERRUPTED_RUN_REPEATS_CALLS``): three nodes each
   calling a model, the run dropped while its third call waits. Resumed from
   the last record it handed over, it makes one call, for the interrupted
   activation, and completes with the result an uninterrupted run gives, whose
   lineage holds the identifiers the first half recorded.
