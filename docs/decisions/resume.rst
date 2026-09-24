==============================
Decisions about resuming a run
==============================

How a run is written down so that it survives the process running it, and how
it is taken up again. ``DEC_RUN_IS_DRIVEN`` left how a run survives a restart
open and argued that a waiting run needs no redesign to allow it;
``DEC_IDENTIFIER_NAMES_ONE_CONTEXT`` left how identifiers survive a process to
this feature, and ``CREQ_SOURCE_SOLE_ISSUER`` said that reading one back from
storage would have to be answered here. These settle all three.

Five rest on measurements recorded in ``evidence/resume``. Two are judgements
and say so: where a record is made, and that a record carries the position of
the identifier source rather than something derived from the identifiers.

What they do not settle is named here. Nothing below decides where a record is
stored, how often a caller other than the scripted run takes one, whether a
record is kept after its run ends, or whether it records what each activation
was given for any purpose but checking a resumed run - that last is the run
log's, which does not exist yet.

.. dec:: A run is recorded as its outputs and the inputs each was made from
   :id: DEC_RECORD_IS_OUTPUTS
   :dec_status: superseded
   :decided_on: 2026-09-23
   :supported_by: EVD_RUN_STATE_DERIVABLE, EVD_REPLAY_BY_NAME_ACCEPTS_REWIRING
   :statement: Agconflo shall record a run as its budget, the activations it spent, its arguments, and each output it accepted together with the inputs that output's activation was given.

   Measured, that is all of a run's state (``EVD_RUN_STATE_DERIVABLE``): the
   outstanding activation, if any, is the one the scheduler gives again for the
   same outputs, and whether there is one is the difference between the
   activations spent and the outputs accepted. Recording the activation as well
   would record something that can disagree with the rest and has to be checked
   against it.

   The inputs are not needed to continue the run, and are recorded anyway
   because without them a record cannot tell its own workflow from another:
   replayed by instance name alone, a rewired workflow was accepted
   (``EVD_REPLAY_BY_NAME_ACCEPTS_REWIRING``). They cost one identifier per
   parameter.

   Recording the workflow definition itself was the alternative, and loses
   twice. A definition is read against node types from other documents, so a
   record carrying one would carry them too, or be unreadable without them. And
   equality is the wrong test: a workflow edited downstream of everything the
   run has done - a typo fixed while a run waits for a person - describes the
   same run so far, and the inputs say so where equality would refuse it.

   Superseded by ``DEC_RECORD_HOLDS_EXCHANGES``, which records every model
   call as well, since a call is a point a run can be written down at and it
   lies inside an activation.

.. dec:: A run is resumed by replaying its record through the run
   :id: DEC_RESUME_BY_REPLAY
   :dec_status: superseded
   :decided_on: 2026-09-23
   :supported_by: EVD_REPLAY_BY_NAME_ACCEPTS_REWIRING
   :statement: Agconflo shall resume a run by starting it from its recorded arguments and reporting each recorded output to it in the recorded order, refusing a record whose outputs the run would not have been offered with the recorded inputs.

   Every check a run makes - the wiring, the signature, a second context under a
   held identifier, an output of an undeclared type - then applies to a resumed
   run as it applied to the run that was recorded, because it is the same code
   meeting the same values. A record that no run could have produced is refused
   by the run, and the refusal is the one a caller already knows.

   Building the run's fields from the record directly was the alternative. It
   would restate each of those checks for the record, and one forgotten is a
   record accepted that the run would have refused - a part sharing an
   identifier with a held context among them, which the run was measured
   missing once already (``EVD_RUN_PART_SHARES_IDENTIFIER``).

   The inputs are compared by identifier, and a context under a recorded
   identifier is the one the record holds, so an input compared equal is the
   same value.

   Superseded by ``DEC_RESUME_REPLAYS_CALLS``, which replays a record's
   calls with its outputs.

.. dec:: A record is a TOML document holding each context once by identifier
   :id: DEC_RECORD_IN_TOML
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_NESTED_RECORD_REFUSED, EVD_RECORD_TEXT_EXACT, EVD_TOML_RECORD_KEYS
   :statement: Agconflo shall write a run's record as a TOML document holding each context once in a table keyed by its identifier, with parts named by identifier and every number written as a decimal string.

   Flat because nested was measured unreadable at a depth of 100 in both
   parsers tried (``EVD_NESTED_RECORD_REFUSED``), and a composition's depth has
   no bound. Keyed because the parser then refuses a second context under one
   identifier by itself (``EVD_TOML_RECORD_KEYS``), leaving only a key written in
   another form - ``07`` for ``7`` - to the reader. Numbers as strings because a
   TOML integer stops at ``i64::MAX`` and an identifier, a budget and a count
   of activations do not.

   JSON was the alternative, and measured, it would have done: text came back
   exact through both (``EVD_RECORD_TEXT_EXACT``), and it wrote 1.7 times and
   read 5 times as fast. It loses on what the difference is worth. The core
   already reads and writes TOML through ``toml_edit`` and has no other
   dependency, and at 100,000 contexts the whole difference is a fraction of a
   second against calls measured at 11 to 30 seconds each
   (``EVD_GENAI_REAL_LOCAL_MODEL``). A record is also what a person reads while
   a run waits for them, and TOML writes a model's answer as the lines it is.

.. dec:: A record is written whole each time one is asked for
   :id: DEC_RECORD_WRITTEN_WHOLE
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_NESTED_RECORD_REFUSED
   :statement: Agconflo shall write a run's whole record each time its caller asks for one rather than appending what changed to an earlier record.

   One record is then one self-contained run, and replacing a file with it is
   the whole of a caller's storage: a record either was written or was not.

   Appending was the alternative. It writes each context once rather than once
   per record, which is the difference between linear and quadratic cost over a
   run - and it brings a torn last entry, an order to replay and a partial
   record to refuse or repair, which is where a journal's defects live. Measured,
   the cost it saves is small here: 100,000 contexts wrote in 0.28 s
   (``EVD_NESTED_RECORD_REFUSED``), and a run records once per activation, each
   of which may wait on a model for tens of seconds. A run whose record costs
   more than its activations is the evidence that would reopen this.

.. dec:: A record carries the position of its identifier source
   :id: DEC_RECORD_CARRIES_THE_SOURCE
   :dec_status: accepted
   :decided_on: 2026-09-23
   :statement: Agconflo shall record the position of the identifier source a run's contexts were made from and resume the run together with a source continuing from that position.

   A judgement, argued from what a source promises: never the same identifier
   twice (``CREQ_SOURCE_NO_REPEAT``). A source continuing from where the
   recorded one stood keeps that promise across the interruption, and the
   record is refused if it holds an identifier at or past that position, since
   no source at that position could have issued it.

   Two alternatives lost. Continuing from the highest identifier the record
   holds repeats identifiers the recorded source issued to contexts the run
   never held - one a script made and dropped, or one its caller made for
   itself - and the record cannot know which of those still exist somewhere.
   Identifiers too large to repeat, drawn at random, would change the value
   every caller and every test depends on, for a guarantee the run's own check
   already gives wherever identifiers come from
   (``DEC_IDENTIFIER_NAMES_ONE_CONTEXT``).

.. dec:: A record is made and read in the core, as text
   :id: DEC_RECORD_IN_CORE
   :dec_status: accepted
   :decided_on: 2026-09-23
   :statement: Agconflo shall write and resume a run's record in agconflo-core as text, leaving where the text is kept to the run's caller.

   A judgement with one fact under it. Only the core can make a context
   identifier (``CREQ_SOURCE_SOLE_ISSUER``), and resuming a context needs one,
   so the record is the core's or the identifier stops being unforgeable.

   As text rather than as a file, because the core opens nothing
   (``DEC_RUN_IS_DRIVEN``): a caller writes it where it likes - a file, a
   database, a message to another process - and the core needs no runtime and
   no file system to be tested.

.. dec:: A scripted run hands its caller a record after every output
   :id: DEC_SCRIPTED_RUN_HANDS_RECORDS
   :dec_status: superseded
   :decided_on: 2026-09-23
   :supported_by: EVD_INTERRUPTED_RUN_REPEATS_CALLS
   :statement: Agconflo shall hand the caller of a scripted run the run's record when the run starts and each time the run accepts an output.

   A scripted run is one call from its start to its ending, so its caller has
   no moment between activations at which to ask for a record. Handed one at
   each, a caller keeping the latest loses at most the activation in progress
   when it was interrupted, where measured it lost everything
   (``EVD_INTERRUPTED_RUN_REPEATS_CALLS``).

   At the start as well, because the arguments are a run's first state and a
   run interrupted in its first activation is otherwise begun again by hand.
   Not before each activation, because nothing has changed since the output
   before it.

   Superseded by ``DEC_RECORD_AFTER_EACH_ANSWER``, which hands one over
   after every answered model call as well.
