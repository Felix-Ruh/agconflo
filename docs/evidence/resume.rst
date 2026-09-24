=============================
Evidence about resuming a run
=============================

Measurements that resuming an interrupted run rests on. All were taken on the
same day, at ``4c091f3``: three against the run itself through temporary tests
added to a crate and removed again, and the format measurements in a throwaway
crate outside the repository against ``toml_edit`` 0.25.15 and ``serde_json``
1.0.151, the releases the workspace already pins.

A temporary test proves nothing by passing, so the one that asserted rather
than printed what it found was given a planted wrong assertion and seen to fail
on it.

.. evd:: A run's state was its arguments, its outputs and one count
   :id: EVD_RUN_STATE_DERIVABLE
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: Over 2000 random runs with refused outputs mixed in, activations spent always equalled outputs accepted plus one when an activation was outstanding, and the scheduler re-derived that activation with the very same input contexts.

   A property test over the generator the run's own tests use for well-formed
   workflows, with budgets from 0 to 7, checked the run's fields before and
   after every step, every refusal and every accepted output. The outstanding
   activation was compared with what ``next_activation`` gives for the same
   definition, arguments and outputs: the same instance, the same parameters,
   and inputs that were the same values rather than equal ones.

   A planted wrong invariant - activations equal to outputs alone - failed on
   the first outstanding activation, so the checks ran.

   It follows from the scheduler holding no history, which its module says and
   this measured: the outstanding activation need not be recorded, only whether
   there is one.

.. evd:: An interrupted scripted run kept nothing, and starting again repeated every call
   :id: EVD_INTERRUPTED_RUN_REPEATS_CALLS
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: A scripted run of three model-calling nodes, dropped while its third call waited, left nothing to continue from, and starting it again made all three calls, repeating the two that had been answered.

   The provider was a stub on the loopback interface that answered two
   requests and held the third open, and the run was dropped by a ``select``
   the moment the third arrived. Dropping it was clean, in that nothing
   panicked; the run was started again on the same thread and completed.

   Two answered calls were thrown away. Against the local model measured
   before (``EVD_GENAI_REAL_LOCAL_MODEL``) that is 11 to 30 seconds each, and
   against a paid provider it is money.

.. evd:: Replaying a record by instance name accepted a rewired workflow
   :id: EVD_REPLAY_BY_NAME_ACCEPTS_REWIRING
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: Three outputs recorded under one workflow replayed without refusal under a second binding an instance to a different entry, in the same order, although that instance would now have been given another input.

   Two entry instances ``x`` and ``y``, an instance ``s`` bound to ``x``, and
   ``t`` bound to ``s``. After ``x``, ``y`` and ``s`` had produced, the same
   outputs were reported to a run of the workflow with ``s`` bound to ``y``.
   Every instance was offered in the recorded order and every output was
   accepted. Only a comparison of inputs told the two apart: ``s`` had been
   given context 2 and would now be given context 3.

   So a replay checking which instance activates would resume a run that never
   happened, holding an output made from inputs the workflow no longer gives it.

.. evd:: A composition written inside its parent failed to read back at depth 100
   :id: EVD_NESTED_RECORD_REFUSED
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: A context 100 compositions deep, each written inside its parent, failed to parse in serde_json at its recursion limit and in toml_edit at its maximum depth, while contexts written flat, parts named by identifier, read back at 100000.

   A composition's depth has no bound here, and 100,000 levels are already
   rendered and walked without recursion (``CREQ_VALUE_PARTS_BY_REFERENCE``).
   Written nested, a record of such a context is one neither parser reads.

   Written flat, 100,000 contexts in a chain took, in a debug build at
   optimisation level 1, 0.42 s to write and 0.46 s to read as TOML (7.0 MB),
   and 0.25 s and 0.09 s as JSON (5.9 MB). Keyed by identifier rather than
   listed, the TOML took 0.28 s and 0.32 s (6.5 MB) in a release build.

.. evd:: Both formats kept arbitrary text byte for byte
   :id: EVD_RECORD_TEXT_EXACT
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: Written and read back through toml_edit and through serde_json, 16 chosen strings and 50000 random ones over all of Unicode came back identical, NUL, lone carriage returns, a byte order mark and surrounding whitespace among them.

   The chosen ones were those a format is likeliest to change: the empty
   string, ``\r\n`` and a lone ``\r``, control characters, U+FEFF, U+2028,
   three quotes of either kind, a backslash, a decomposed accent, an emoji,
   U+10FFFF, and an answer with blank lines around it. A comparison against an
   altered string, as a control, reported the difference.

   ``toml_edit`` writes text with line breaks as a multi-line string, so a
   recorded answer reads as the text it is.

.. evd:: TOML refused a repeated key, took 7 and 07 as two, and stopped at i64
   :id: EVD_TOML_RECORD_KEYS
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: toml_edit 0.25.15 refused a document holding one table key twice, written alike or once bare and once quoted, read 7 and 07 as two keys, and refused 18446744073709551615 as an integer while writing 9223372036854775807.

   A context identifier is a ``u64``, and an identifier source issues up to
   ``u64::MAX``, so an identifier written as a TOML integer cannot hold the top
   half of the range. Written as a key, one identifier names one table, and the
   parser refuses the second rather than keeping either - except that a key is
   text, so ``7`` and ``07`` are two tables for one identifier unless the reader
   refuses the form that is not canonical.

The last was taken the next day, once the record existed, and is the one
measurement here of a process actually ending: against the local model the
models feature was measured on (``EVD_GENAI_REAL_LOCAL_MODEL``), from a scratch
binary built on the crates at the head of the implementation.

.. evd:: A run killed during a model call resumed in a new process with one call
   :id: EVD_RESUMED_ACROSS_A_RESTART
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: A three-node scripted run against qwen3.8-27b-ridge, its process killed during the third model call, resumed from its last record in a new process, kept all five recorded contexts byte for byte under their identifiers and made two new ones.

   Each node asked the model for an animal and a four-line poem, and the
   binary replaced a record file with every record it was handed. When the
   file held two outputs, 54 s in, the process was killed outright within half
   a second - after the record, which the loop hands over just before it
   starts the next activation's call, and seconds short of any answer, going
   by the calls either side. A new process read the file, resumed, and
   completed in 23.9 s, against 6 s and 53 s for the first two calls.

   The record it ended with held every context of the one it began with,
   identical, and two more: one answer and the result. A call makes one
   answer, so one call was made - for the activation that was killed - and
   the activations spent went from 2 to 3, not 4. The two answers the killed
   process was given are in the result as it was given them.
