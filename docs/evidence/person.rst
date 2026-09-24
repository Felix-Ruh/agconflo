================================
Evidence about a person in a run
================================

Measurements that having a person take part in a run rests on. All were taken
on the same day, at ``5dac701``, through temporary tests added to a crate and
removed again: three through ``agconflo-core``'s public interface alone, as an
outside caller would drive a run, and two through ``agconflo-lua``'s scripted
run.

No person took part. Where one would supply a context, the test supplied a
fixed text in their place, which is all a person's answer is to the run - a
context arriving from outside it.

A temporary test proves nothing by passing, so the one that asserted rather
than printed what it found was given two planted wrong assertions, one on the
record and one on the result's lineage, and seen to fail on each.

.. evd:: A caller driving the core alone had a person perform an activation across a restart
   :id: EVD_CORE_PARKS_FOR_A_PERSON
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: A run parked on one activation, recorded, dropped and resumed from the text alone took a context supplied for that activation and completed holding it in its result's lineage, having spent one activation per node.

   A chain of three node types: ``draft``, then ``review`` bound to it with an
   output of type ``verdict``, then ``publish`` bound to both. The caller
   performed ``draft`` itself, stopped at ``review``, and recorded the run -
   once after asking for the next activation and once before, since a caller
   may keep either. Everything but the text was dropped.

   From either record, the resumed run offered ``review`` with the same input
   under the same identifier and asked for a ``verdict``. A context made from the
   source that came back with the run was accepted, and the run completed with
   three activations spent for three nodes. Asking the parked run for its next
   activation a second time gave the same one and left its record unchanged
   byte for byte.

   Held on the last activation its budget allowed, a resumed run offered that
   activation again rather than ending, accepted its output, and then ended on
   its budget.

   So the core needs nothing for a person to take part, which is what
   ``DEC_RUN_IS_DRIVEN`` argued without measuring: a run that is waiting is a
   run nobody has fed.

.. evd:: A scripted run refused a workflow with a step no script performs
   :id: EVD_SCRIPTED_RUN_REFUSES_A_PERSON
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: A scripted run of a three-node chain, its later two nodes of a type the caller had given no script because a person was to perform it, was refused before starting with the fault that the type has no script.

   The refusal is the one ``CREQ_BEHAVIOURS_REFUSE_MISSING`` requires, and it is
   right for what it was written against. It also means that the one caller in
   this project that drives a run from start to end cannot drive a run a person
   takes part in, whatever the core allows.

.. evd:: Work done inside an interrupted activation was done again
   :id: EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: A script making two model calls, its run dropped while the second was held open, left a record of no outputs, and resumed it made both calls again, the one already answered among them.

   The provider was a stub on the loopback interface that answered one request
   and held the next open, and the run was dropped the moment the second
   arrived. A record is taken between activations, never inside one, so what an
   activation had done when it was interrupted is not in it.

   A person asked for something from inside a script would be in the held
   call's place, for hours rather than seconds. A restart in that window would
   throw away everything the activation had done before asking, and the
   question itself, since the answer has nowhere to go.

.. evd:: A run with one activation outstanding offered no other
   :id: EVD_PARKED_ACTIVATION_HOLDS_OTHER_BRANCHES
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: With two instances ready at once in a fork, the run offered the one written first, and asked again before that one had produced, it offered the same one again rather than the other.

   Two instances bound to one entry instance, joined by a fourth. Written in
   one order the first ready instance offered was one, written in the other
   order it was the other, which is the scheduler scanning the definition.

   A run hands out one activation at a time (``DEC_RUN_IS_DRIVEN``) and holds
   it until it is answered (``DEC_REFUSED_OUTPUT_OUTSTANDING``). So while a
   person performs one activation, an instance on another branch that a script
   could run at once waits for the person, however long that takes. What the
   run ends with does not depend on it; when it ends does.

.. evd:: One record answered twice gave one identifier to two contexts
   :id: EVD_ONE_RECORD_ANSWERED_TWICE
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: A run recorded while it awaited a context, resumed twice from the one record and given a different context each time, issued the same identifier to both, so two runs went on, each sound, sharing an identifier.

   Each resumed run was a correct run on its own and every check it makes
   passed. Nothing in either could see the other: a resumed run is built from
   its record's text and nothing else, and the core stores nothing
   (``DEC_RECORD_IN_CORE``).
