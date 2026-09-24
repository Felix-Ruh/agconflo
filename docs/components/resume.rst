============================
Components of resuming a run
============================

The component ``ARCH_RESUME`` adds, and the requirements allocated to it and to
the script host for resuming a run. Each title is the grammatical subject of
the requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

One requirement here derives from a feature of ``features/run`` rather than of
``features/resume``: that a resumed run's identifier source repeats nothing is
the run's rule of one context per identifier, kept across an interruption.

.. comp:: Run record
   :id: COMP_RUN_RECORD
   :crate: agconflo-core

   A run written down as text, and a run taken up again from the text. It
   writes what a run holds and reads it back into a run that holds the same,
   together with an identifier source that continues where the recorded one
   stood.

   It resumes a run by starting one and reporting the recorded outputs to it
   (``DEC_RESUME_BY_REPLAY``), so what it adds to the run's own checks is what
   only a record can get wrong: text that is not a record, contexts that do not
   hold together, and a history the workflow would not have produced.

.. comp_req:: A record holds everything a run holds
   :id: CREQ_RECORD_HOLDS_THE_RUN
   :derived_from: FEAT_RUN_RECORDED
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: ubiquitous
   :statement: Run record shall write a run as text holding its budget, the activations it spent, its arguments, each output it accepted in the order accepted with the inputs its activation was given, and every context those hold once each.

   Measured, those are the whole of a run's state (``EVD_RUN_STATE_DERIVABLE``),
   and the inputs are what tells its workflow from another
   (``DEC_RECORD_IS_OUTPUTS``).

   Failure modes:

   - **The activations spent left out.** A run recorded with an activation
     outstanding resumes with none, and spends its budget on it a second time.
   - **The inputs left out.** A rewired workflow resumes the run
     (``EVD_REPLAY_BY_NAME_ACCEPTS_REWIRING``).
   - **A composition written inside its parent.** A deep one is written and
     never read back (``EVD_NESTED_RECORD_REFUSED``).
   - **A context written once per holder.** A part held by two contexts comes
     back as two values under one identifier.

.. comp_req:: A resumed run continues where it was recorded
   :id: CREQ_RECORD_CONTINUES_THE_RUN
   :derived_from: FEAT_RESUME_REPEATS_NO_OUTPUT
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: ubiquitous
   :statement: Run record shall resume a run holding every output its record holds, having spent the activations its record spent, and offering again the activation that was outstanding when it was recorded.

   Failure modes:

   - **The outstanding activation charged again.** A run with a budget exactly
     large enough stops short of completing after being resumed.
   - **The outstanding activation lost.** The resumed run offers the next one,
     and the instance that was running never produces.
   - **The recorded outputs handed to the caller to perform again.** Every
     recorded activation runs twice, which is the measured shape
     (``EVD_INTERRUPTED_RUN_REPEATS_CALLS``).

.. comp_req:: A resumed context is the context recorded
   :id: CREQ_RECORD_KEEPS_CONTEXTS
   :derived_from: FEAT_RESUME_KEEPS_CONTEXTS
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: ubiquitous
   :statement: Run record shall resume each context its record holds with the identifier, declared type, text or parts and separator it was written with, as one value however many contexts hold it.

   Failure modes:

   - **Text changed on the way.** Line endings converted, whitespace trimmed or
     characters escaped and not unescaped: the resumed run holds bytes nobody
     produced (``EVD_RECORD_TEXT_EXACT``).
   - **A part made once per holder.** Two values under one identifier, which the
     run refuses (``DEC_IDENTIFIER_NAMES_ONE_CONTEXT``), so a sound record is
     refused.
   - **A deep composition built by recursion.** 100,000 levels were measured
     aborting a recursive render; a recursive resume aborts the same way.

.. comp_req:: A resumed run's source repeats nothing the recorded one issued
   :id: CREQ_RECORD_SOURCE_CONTINUES
   :derived_from: FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: ubiquitous
   :statement: Run record shall resume a run together with an identifier source that issues no identifier the recorded run's source had issued.

   The recorded source's position is part of the record
   (``DEC_RECORD_CARRIES_THE_SOURCE``), and an exhausted source comes back
   exhausted.

   Failure modes:

   - **A fresh source.** Its first identifier is one the run holds, and every
     output made from it is refused (``CREQ_RUN_REFUSES_HELD_IDENTIFIER``).
   - **A source continuing from the highest identifier held.** An identifier
     issued and not held - a context a script made and dropped - is issued
     again.
   - **An exhausted source resumed as fresh.** Its first identifier repeats the
     first ever issued.
   - **A record naming an identifier its source had not reached.** Resumed,
     the source then issues it a second time. So a record holding an
     identifier at or past its source's position is refused, and one whose
     source is exhausted resumes with a source that issues nothing. Moved here
     from ``CREQ_SOURCE_SOLE_ISSUER`` by ``DEC_CHANGE_SOURCE_SOLE_ISSUER``.

.. comp_req:: A record the workflow would not have produced is refused
   :id: CREQ_RECORD_REFUSES_DIVERGENCE
   :derived_from: FEAT_RESUME_REFUSES_ANOTHER_RUN
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: unwanted
   :statement: If the workflow a record is resumed against would not have offered each recorded output's activation in the recorded order with the recorded inputs, then Run record shall refuse to resume it naming the first recorded output that differs.

   The run offers each activation in turn and is handed the recorded output
   (``DEC_RESUME_BY_REPLAY``); what the run offers is compared first.

   Failure modes:

   - **Only instance names compared.** The measured shape: a rewired workflow
     resumes (``EVD_REPLAY_BY_NAME_ACCEPTS_REWIRING``).
   - **Recorded outputs left over.** A record holding more outputs than the
     workflow would have asked for - one after its designated output, or one
     for an instance it does not have - resumes with some of them never
     reported.
   - **An output the run refuses reported anyway.** A recorded output of a type
     the workflow no longer declares for its instance is held by the resumed
     run.
   - **More activations spent than a run could have spent.** A count below the
     outputs, or more than one above them, describes no run.

.. comp_req:: A record whose run would not start is refused as the start refuses
   :id: CREQ_RECORD_REFUSES_WHAT_START_REFUSES
   :derived_from: FEAT_RESUME_REFUSES_ANOTHER_RUN
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: unwanted
   :statement: If a run of the workflow a record is resumed against would be refused its start with the record's arguments, then Run record shall refuse to resume it carrying that refusal.

   Failure modes:

   - **The refusal reported as a divergence.** A wiring defect in the workflow
     is a fault a caller fixes in the workflow, and it arrives looking like a
     fault in the record.
   - **The start's checks skipped.** A defective workflow is resumed, which no
     run of it could have been.

.. comp_req:: A text that is not a record is refused with its fault
   :id: CREQ_RECORD_REFUSES_UNREADABLE
   :derived_from: FEAT_RESUME_REFUSES_ANOTHER_RUN
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: unwanted
   :statement: If a text cannot be read as a run record, then Run record shall refuse to resume it naming the fault and where in the text it is.

   Where is a line and a column for every fault, since every value of a parsed
   record carries its place, and the key the fault is under for every fault but
   text that is not TOML at all.

   Failure modes:

   - **Any TOML accepted.** A record of another version, or with a field
     missing, resumes with a default in its place.
   - **An unknown field ignored.** A record written by something else is read
     as if it were one of these, minus whatever that field said.
   - **An identifier written in another form.** ``07`` beside ``7`` is two
     contexts under one identifier (``EVD_TOML_RECORD_KEYS``).
   - **A part naming no recorded context, or a composition holding itself.**
     Resumed, the first has nothing to point to and the second never finishes
     being built.
   - **An identifier at or past the recorded source's position.** No source at
     that position could have issued it, so the record is not one run's.
   - **A context nothing holds, kept.** A record holds what its run held, and a
     context no argument, output or part reaches was written by something
     else; resumed, it would be dropped without a word.

.. comp_req:: A scripted run hands over its record at the start and after each output
   :id: CREQ_HOST_HANDS_RECORDS
   :derived_from: FEAT_SCRIPTED_RUN_RECORDED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a scripted run starts or accepts an output, Script host shall hand the run's caller a record taken after that output was accepted and holding the position of the source the run's scripts draw from.

   Failure modes:

   - **Handed only at the end.** The measured shape: an interrupted run leaves
     nothing (``EVD_INTERRUPTED_RUN_REPEATS_CALLS``).
   - **Handed before the output is accepted.** A record holding an output the
     run then refused describes a run that never happened.
   - **The position read from the caller's source while it is lent.** What
     stands in its place during a run is a fresh source, so the record claims
     a position of zero and every identifier it holds is past it.
