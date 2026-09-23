=================================
Evidence about running a workflow
=================================

Findings from a prototype run, built to answer one question before any
requirement was written: is a run expressible as a state machine its caller
drives, over the ``WorkflowDefinition`` ``agconflo-core`` already has? It is,
and building it turned up four behaviours that no requirement had ruled out.

These are ``prototype`` rather than ``measurement``, and the distinction is how
each could be re-checked. The evidence about storage measures libraries, and
re-checking one means running that library again. These measure a shape this
project invented, so re-checking one means building that shape again - which is
what the descriptions below are for.

The prototype was a throwaway crate outside the repository, depending on
``agconflo-core`` by path at ``9118dec``. It held the run state in a map from
instance name to the context that instance had produced, handed the caller one
activation at a time, and took back either a produced context or a failure. An
instance was taken to be ready when every parameter its type declared
``required`` had a context, from the run's arguments for an entry instance and
from a binding's source otherwise. Each finding below is a run of that crate
over a definition written for it, and every one of those definitions passed
``validate_wiring`` with no defect reported, which is why these are findings
about running rather than about wiring.

.. evd:: An optional input arrived or did not by where its producer was written
   :id: EVD_RUN_OPTIONAL_BY_ORDER
   :evd_kind: prototype
   :observed_on: 2026-09-22
   :observation: A prototype run gave a consumer its optional input when the producer was written above it and left that parameter unfilled when the producer was written below it, from one definition and one set of arguments.

   Three instances: two entry nodes, ``fast`` and ``slow``, and a consumer
   ``sink`` whose type declares ``must`` as required and ``may`` as optional,
   bound to ``fast`` and ``slow`` respectively.

   With the instances written ``fast``, ``slow``, ``sink`` the activations were
   ``fast(seed)``, ``slow(seed)``, ``sink(must+may)``. Moving ``sink`` one line
   up, so that the order read ``fast``, ``sink``, ``slow``, the activations
   became ``fast(seed)``, ``sink(must)`` - the same definition, the same
   arguments, and a node given less than the wiring says reaches it.

   The cause is that readiness was asked of the required parameters alone, so
   ``sink`` became ready the moment ``must`` had a value and was found by a scan
   in the order the definition carries its instances. Nothing reported it. The
   run completed and the output was well formed, which is the shape of wrong
   answer this project exists to rule out: what a node saw was decided by where
   a line sat in a document rather than by what was wired to it.

.. evd:: Two entry instances sharing a parameter name were handed one context
   :id: EVD_RUN_ENTRY_NAME_SHARED
   :evd_kind: prototype
   :observed_on: 2026-09-22
   :observation: Two entry instances whose node types each declared a parameter named seed, of context types Seed and Other, were both handed the same context by a run holding its arguments under parameter names alone.

   The prototype held a run's arguments as a map from parameter name to
   context, which is the obvious reading of a workflow's parameter list being
   the parameters its entry instances declare.

   Two entry instances, ``p`` of type ``Alpha`` and ``q`` of type ``Beta``, each
   declaring one required parameter called ``seed``. ``Alpha`` declares it for
   context type ``Seed`` and ``Beta`` for ``Other``. One argument was supplied,
   of type ``Seed``, and both instances received it. ``q`` was given a context of
   a type its own declaration does not name, and the run completed.

   ``validate_wiring`` reported nothing, correctly: the definition's wiring is
   sound, and an entry instance's parameters are not wired at all. The name
   collision is a property of the workflow's signature rather than of its
   wiring, and nothing looks at the signature yet.

.. evd:: A missing argument was indistinguishable from a deadlock
   :id: EVD_RUN_MISSING_ARGUMENT_QUIESCES
   :evd_kind: prototype
   :observed_on: 2026-09-22
   :observation: A prototype run of a two-node workflow started with no arguments reported the same quiescence, naming both instances as waiting, as a run of two instances bound to each other in a cycle.

   Started with no arguments, the linear workflow ``a -> b`` reported
   ``Stuck { waiting: ["a", "b"] }``. Two instances bound to each other's output,
   with no entry node between them, reported ``Stuck { waiting: ["c", "d"] }``.
   The two readings differ only in the names.

   They are not the same kind of thing. The cycle is a property of the
   definition, and quiescence is the honest way to find it - nothing is running
   and nothing can be made ready. The missing argument is a fault in how the run
   was started, known before any instance was looked at, and reporting it as a
   run that got stuck names every waiting instance instead of the one thing the
   caller has to fix.

.. evd:: A run completed while holding an instance that could never activate
   :id: EVD_RUN_COMPLETES_WITH_IDLE
   :evd_kind: prototype
   :observed_on: 2026-09-22
   :observation: A prototype run whose definition held an instance bound to its own output, which can never become ready, completed as soon as the designated output had a value and never reported that instance.

   The definition held ``live``, an entry instance, and ``stuckone``, whose only
   required parameter is bound to ``stuckone`` itself. The designated output was
   ``live``. The run completed on ``live``'s activation and said nothing about
   ``stuckone``, which had not run and never could.

   Recorded because it looks like a defect and is not one.
   ``DEC_ROUTING_SEPARATE_FROM_CONTROL`` keeps a context's path apart from which
   nodes run, so an instance that no route to the designated output passes
   through has no claim to be activated, and a run that waited for it would be
   reporting a stuck run on a workflow that had produced its result. What the
   finding does settle is that completion is a question about the designated
   output alone, and therefore that quiescence is reported only when that output
   can no longer be produced.

The two findings below are a different kind of evidence about the same run, and
they are ``measurement`` rather than ``prototype``: each is a run of
``agconflo-core`` itself at ``4dc940b``, driven by a throwaway caller outside the
repository that answered each activation with a context chosen to contradict
one declaration. Both definitions passed ``validate_wiring`` with nothing
reported, and every run below completed. They were taken while planning node
behaviour, because a script is the first caller that can return anything at
all.

.. evd:: A run accepted an output of a type its node type does not declare
   :id: EVD_RUN_ACCEPTS_UNDECLARED_OUTPUT
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: A run of agconflo-core at 4dc940b accepted an output of context type banana from an instance whose node type declares prompt, handed it to a parameter declared for prompt, and completed.

   Two instances: an entry instance ``a`` of a type declaring the output
   ``prompt``, and ``b``, whose one required parameter is declared for
   ``prompt`` and bound to ``a``. The caller answered ``a``'s activation with a
   new context of type ``banana``. ``Run::produced`` accepted it, ``b``'s
   activation carried it under the parameter declared for ``prompt``, and the
   run completed on ``b``.

   ``validate_wiring`` had compared ``a``'s declared output with ``b``'s
   declared parameter and found them equal, which was true of the declarations
   and false of the run. Nothing compares what an instance produces with what its
   type declares, so the check made before a run starts holds only for as long
   as every caller happens to honour the declarations it checked.

.. evd:: A run accepted an output carrying an identifier it already held
   :id: EVD_RUN_ACCEPTS_HELD_IDENTIFIER
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: A run of agconflo-core at 4dc940b accepted as an instance's output first the argument that instance was given and then another instance's output, each carrying the identifier it already had, and completed both times.

   The same two instances, with ``b``'s type declaring the output ``prompt`` so
   that no type disagreed. In one run the caller answered ``a``'s activation
   with the argument ``a`` had been given, identifier 0. In another it answered
   ``b``'s activation with ``a``'s output, identifier 1, and that run completed
   with identifier 1 as its result - the context ``a`` produced, recorded as
   ``b``'s.

   ``DEC_IDENTITY_PER_ACTIVATION`` gives every context an identifier unique to
   the activation that produced it, and here one identifier is credited to two
   activations, or to an activation and the caller. A control in the same
   session answered ``b``'s activation with a composition of its input instead.
   That was accepted with a new identifier, 2, holding the input by reference,
   which is the shape a node passing its input on is meant to take.
