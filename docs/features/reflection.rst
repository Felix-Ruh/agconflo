=======================================
The engine explaining itself to a model
=======================================

What a model is told by the engine about what it asked of it. Every
requirement here derives from ``STKH_REFLECTION``, and each is written
against the decisions in ``decisions/reflection``.

.. feat_req:: A model is told why a call it made was refused, and goes on
   :id: FEAT_REFUSAL_GIVEN_TO_MODEL
   :derived_from: STKH_REFLECTION
   :ears_pattern: event
   :verification_method: test
   :statement: When Agconflo refuses a call a model made, Agconflo shall give the model a context saying why in place of that call's output and continue the model's activation.

   The parent at the point where a model most needs the engine to explain
   itself: the model called something through the interface it was given,
   and the engine would not do it. A model told why can call again as it
   should; a model whose activation ends there learns nothing, and neither
   does the run, which loses the activation's work and everything after it.

   It can be false while the parent holds only in its letter: an interface
   that is defined and documented, but answers a call it refuses by ending
   the caller's activation, is one the model cannot ask what it did wrong.
   The parent asks for an engine that explains itself to a model, and a
   refusal is the engine's own act.

   It claims no more than the parent. Which calls are refused is
   ``FEAT_YIELD_UNDECLARED_REFUSED``'s; what the context says, what type it
   has, and what becomes of the other calls of the same answer are
   decisions (``DEC_REFUSED_CALL_ANSWERED``,
   ``DEC_UNREFUSED_CALLS_PERFORMED``).

.. feat_req:: A resumed activation is given the refusals its record holds
   :id: FEAT_REFUSAL_RESUMED
   :derived_from: STKH_REFLECTION, STKH_RESUMABLE_RUN
   :ears_pattern: event
   :verification_method: test
   :statement: When a run is resumed whose record holds a call that was refused, Agconflo shall give the resumed activation the same refusal for that call without asking the model again.

   ``STKH_RESUMABLE_RUN`` resumes an interrupted run, and a resumed
   activation answers each model call its record holds from the record. A
   refused call is part of what the model was answered with, so a resume
   that could not give it again would ask the model again, for an answer
   paid for once already, or would send the model turns different from the
   ones it answered.

   It can be false while both parents hold: an engine that tells a model
   why and resumes runs, but keeps no refusal in its record, resumes an
   activation as far as its first refused call and no further.

.. feat_arch:: A refusal is made by the script host, kept by the run and its record, and sent by the model roster
   :id: ARCH_REFUSAL_ANSWERED
   :realises: FEAT_REFUSAL_GIVEN_TO_MODEL, FEAT_REFUSAL_RESUMED
   :uses: COMP_SCRIPT_HOST, COMP_WORKFLOW_RUN, COMP_RUN_RECORD, COMP_MODEL_ROSTER
   :statement: Agconflo shall allocate telling a model why a call was refused to the script host, the workflow run, the run record and the model roster.

   - The script host refuses a call it cannot make into one, and answers
     that call, or one the run refused, with its refusal, which is part of
     the next window.
   - The workflow run holds each refused call an exchange reports, with its
     contexts.
   - The run record writes each refused call and reads it back.
   - The model roster sends the model's turn back with the refused call as
     the model made it, and the refusal as that call's result.

   The runner and the command line are not among them: a refusal changes
   no ending of a run, and the record is kept as before, whole.
