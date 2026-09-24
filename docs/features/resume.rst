==============
Resuming a run
==============

A run written down while it is under way, and taken up again from what was
written - in another process, after the one running it has gone. Every
requirement here derives from a goal in ``stakeholder/execution`` or
``stakeholder/context``, and each is written against the decisions in
``decisions/resume``.

What resuming does not do here is what keeps this a slice. Nothing stores a
record: the core hands its caller text, and where the text is kept is the
caller's (``DEC_RECORD_IN_CORE``). No person takes part in a run in this slice,
so the wait that makes ``STKH_RESUMABLE_RUN`` load-bearing is not built here,
only what makes surviving one possible; ``features/person`` builds it. And a
resumed run is the run it was: no output it recorded is revisited, and nothing about its workflow can be changed
that would have changed what it already did.

Each requirement was checked by hand against the question no rule can ask:
could this be false while its parent is true? The body of each says how. Two
statements are ``ubiquitous``, two are ``event`` and one is ``unwanted``.

The feature's architecture closes the file. It realises all five requirements
and names the components they are divided between, which are defined in
``components/resume``.

.. feat_req:: A run can be written down while it is under way
   :id: FEAT_RUN_RECORDED
   :derived_from: STKH_RESUMABLE_RUN
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall give the caller of a run a record of that run as text whenever the caller asks for one.

   The parent resumes a run that was interrupted, and a run that is to survive
   its process has to exist somewhere outside it.

   It can be false while the parent holds, read loosely. A run held in memory is
   resumed perfectly well after an interruption the process survives - a caller
   that stopped asking for activations and started again. The parent's body
   names the case that matters, a process restart while a run waits, and that
   one needs the run written down.

.. feat_req:: A resumed run does no recorded work again
   :id: FEAT_RESUME_REPEATS_NO_OUTPUT
   :derived_from: STKH_RESUMABLE_RUN
   :ears_pattern: event
   :verification_method: test
   :statement: When a run is resumed from its record, Agconflo shall continue it without performing again any activation whose output the record holds.

   The parent's own words, made testable: work is thrown away exactly when it
   is performed a second time. Measured, today every activation is
   (``EVD_INTERRUPTED_RUN_REPEATS_CALLS``).

   It can be false while the parent holds. A run started again from its
   arguments has, in a sense, been resumed: it reaches the ending it would have
   reached, having repeated every model call on the way. The test is that shape
   exactly, with the calls counted.

.. feat_req:: A resumed run holds the contexts it held
   :id: FEAT_RESUME_KEEPS_CONTEXTS
   :derived_from: STKH_PROVENANCE
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall resume a run holding every context its record holds under the identifier, declared type and content that context was recorded with.

   The parent records which context each byte of a node's input came from, and
   every answer to that question is given by identifier.

   It can be false while the parent holds. A resumed run whose contexts were
   made afresh from the recorded text - new identifiers, the same bytes - goes
   on to a correct result, and every lineage it reports is true of the resumed
   run and false of the run that was recorded: an identifier held in a record
   of the first half names nothing in the second.

.. feat_req:: A record of another run is refused
   :id: FEAT_RESUME_REFUSES_ANOTHER_RUN
   :derived_from: STKH_RESUMABLE_RUN
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a record does not describe a run of the workflow it is resumed against, then Agconflo shall refuse to resume it naming the fault it carries.

   The parent resumes the run that was interrupted, not a run.

   It can be false while the parent holds, and was measured so: a rewired
   workflow accepted every recorded output, in order
   (``EVD_REPLAY_BY_NAME_ACCEPTS_REWIRING``), and the run it continued held an
   output made from an input its workflow no longer gives. A record that cannot
   be read, or that no run could have left, is the same fault from the other
   side: it describes no run at all.

.. feat_req:: A scripted run hands its caller a record as it goes
   :id: FEAT_SCRIPTED_RUN_RECORDED
   :derived_from: STKH_RESUMABLE_RUN
   :ears_pattern: event
   :verification_method: test
   :statement: When a scripted run starts or accepts an output, Agconflo shall hand the run's caller the run's record.

   A scripted run is one call from start to ending, and its caller has no
   moment between activations to ask for a record in
   (``DEC_SCRIPTED_RUN_HANDS_RECORDS``).

   It can be false while the parent holds. A scripted run that hands its record
   over when it ends is resumable, and never needs to be: an interrupted one
   never ends.

.. feat_arch:: Resuming a run splits into a run record and the script host
   :id: ARCH_RESUME
   :realises: FEAT_RUN_RECORDED, FEAT_RESUME_REPEATS_NO_OUTPUT, FEAT_RESUME_KEEPS_CONTEXTS, FEAT_RESUME_REFUSES_ANOTHER_RUN, FEAT_SCRIPTED_RUN_RECORDED
   :uses: COMP_RUN_RECORD, COMP_SCRIPT_HOST
   :statement: Agconflo shall allocate resuming a run to the run record and the script host.

   Two components, each answerable for what the other cannot guarantee:

   - The run record answers for the text: what it holds, what a run resumed
     from it holds, which identifier source comes back with it, and every
     reason one is refused. It lives in the core, because only the core can
     make an identifier (``DEC_RECORD_IN_CORE``).
   - The script host answers for when a scripted run hands a record over, which
     is a statement about the loop performing activations rather than about the
     text.

   The workflow run is not a third. A resumed run is a run started and fed its
   recorded outputs (``DEC_RESUME_BY_REPLAY``), so every requirement of
   ``components/run`` holds of it unchanged, and none is added.

   The decisions this is built against are named here rather than linked:

   - ``DEC_RECORD_IS_OUTPUTS``: a run is recorded as its outputs and the inputs
     each was made from, and its outstanding activation is re-derived.
   - ``DEC_RESUME_BY_REPLAY``: a record is resumed through the run's own checks.
   - ``DEC_RECORD_IN_TOML``: contexts are held flat, keyed by identifier, with
     numbers as strings.
   - ``DEC_RECORD_WRITTEN_WHOLE``: each record is complete on its own.
   - ``DEC_RECORD_CARRIES_THE_SOURCE``: a resumed run's source continues where
     the recorded one stood.
   - ``DEC_RECORD_IN_CORE``: the core writes and reads text and stores nothing.
   - ``DEC_SCRIPTED_RUN_HANDS_RECORDS``: at the start and after every output.
