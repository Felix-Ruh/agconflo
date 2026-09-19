==================
Validating a graph
==================

The engine's first feature: deciding whether a workflow is well formed, before
anything in it runs. Every requirement here derives from a goal in
``stakeholder/execution`` or ``stakeholder/authoring``, and each is written
against the decisions in ``decisions/workflow`` rather than re-opening them.

Four of them name a defect class and say the workflow is refused for it. They
are separate requirements rather than one, because a validator that catches
three of the four is a validator that reports success on a workflow that cannot
run - and "invalid" in the parent goal is exactly the word that has to be spelled
out somewhere.

The other two are about the report rather than the verdict. An agent is a first
class author here, and an agent correcting one defect per round trip is a
different tool from one that sees them all at once.

Each was checked by hand against the question no rule can ask: could this be
false while its parent is true? The body of each says how.

Deliberately not here, and worth naming so that their absence is a decision
rather than an oversight: a node that no entry node can reach is not refused,
because a half-wired workflow is the ordinary state of one being authored and
nothing about it prevents a run; and a cycle is not a defect at all
(``DEC_BACK_EDGES_ALLOWED``), so nothing here looks for a topological order.

.. feat_req:: A required parameter must be bound
   :id: FEAT_WIRING_REQUIRED_BOUND
   :derived_from: STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a workflow leaves a required parameter of a node unbound, then Agconflo shall reject that workflow before running any node of it.

   The defect the whole feature exists for. A node type declares what it requires
   (``DEC_DECLARED_PARAMETERS``), and a workflow that does not satisfy that
   declaration describes a node that cannot be activated.

   It can be false while its parent holds, which is why it is written down: a
   validator that resolves every binding and checks every type still says nothing
   about a parameter that has no binding to resolve. Absence is the case that
   loop-over-what-is-there never visits, and it is the commonest authoring
   mistake there is.

   Optional parameters are deliberately outside it. A node usable with less than
   everything wired is the reason the declaration has two lists rather than one.

.. feat_req:: A binding must name something that exists
   :id: FEAT_WIRING_BINDING_RESOLVES
   :derived_from: STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a binding names a node instance or an output that the workflow does not contain, then Agconflo shall reject that workflow.

   A binding names a position in the graph rather than a value
   (``DEC_BINDING_BY_PORT``), so both of its ends can be resolved without running
   anything - and a name that resolves to nothing is a wire to nowhere.

   It can be false while its parent holds and while the requirement above holds
   too: every required parameter has a binding, each binding is present and well
   formed, and one of them names an instance that was renamed or deleted. The
   parameter is bound, the workflow is broken, and counting bindings finds
   nothing wrong.

   A missing node *type* is the same defect one layer up (``DEC_TWO_LAYERS``):
   an instance of a type the workflow does not carry has nothing to be checked
   against, so it is refused here rather than discovered when the node is asked
   to run.

.. feat_req:: A binding must agree on the context type
   :id: FEAT_WIRING_TYPES_AGREE
   :derived_from: STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a binding joins an output to a parameter declared for a different context type, then Agconflo shall reject that workflow.

   Contexts are nominally typed, and one of the three jobs that typing does is
   static validation of wiring. This is that job made observable.

   It can be false while both requirements above hold, and the failure is silent
   rather than loud: every parameter is bound, every name resolves, and a node
   built to read a diff is handed a summary. Nothing crashes. The node assembles
   its inputs by type (``DEC_DECLARED_PARAMETERS``) and produces a confident
   wrong answer, which is worse than a refusal and far harder to find.

   The check is possible before a run only because the type travels on the
   declaration rather than on the value, which is what ``DEC_CONTEXT_TYPING``
   settles for the value side.

.. feat_req:: A workflow must designate exactly one output
   :id: FEAT_WIRING_ONE_DESIGNATED_OUTPUT
   :derived_from: STKH_WORKFLOW_AS_NODE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a workflow designates no output or designates more than one, then Agconflo shall reject that workflow.

   A workflow is invocable as a node, and a caller wires to its signature
   (``DEC_WORKFLOW_SIGNATURE``). A signature missing its result is not something
   a caller can bind to; two results would make the one-output rule false at the
   composite level while holding at every node inside it.

   It can be false while its parent holds, and the way it can is worth stating:
   a workflow can be invoked perfectly well while the engine picks a terminal
   node itself - by wiring, by declaration order, by whatever is last. Invocation
   works. What breaks is that the workflow's result is decided by the engine
   rather than declared by its author, so adding a second terminal node silently
   changes what callers receive.

   It is checked here rather than at invocation because a workflow with no
   declared result is malformed whether or not anything ever calls it.

.. feat_req:: Every defect is reported together
   :id: FEAT_WIRING_ALL_DEFECTS
   :derived_from: STKH_MACHINE_AUTHORING
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall report every wiring defect of a rejected workflow rather than the first one found.

   The parent requires that errors be reported well enough for a model to correct
   itself from them. One defect per attempt satisfies the letter of that and
   wastes the loop: ten defects become ten cycles of edit, revalidate, read, each
   costing a model call.

   It can be false while its parent holds. A single defect can be reported
   perfectly - named, located, explained - and the validator can still stop at it.
   The report is good; the tool is slow, and the author is a machine that pays
   per round trip.

   It also constrains the implementation rather than only the message: a
   validator that returns on the first defect cannot be made to satisfy this by
   improving its wording.

.. feat_req:: A defect says where it is
   :id: FEAT_WIRING_DEFECT_LOCATED
   :derived_from: STKH_MACHINE_AUTHORING
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall name the node instance and the parameter that each reported wiring defect concerns.

   The other half of a correctable report, and separable from the half above. A
   validator can report all ten defects and describe each as a type mismatch
   without saying which wire it is about, leaving the author to search the graph
   for the ten places it might be.

   It can be false while its parent holds for the same reason its sibling can: the
   errors are reported, and what makes them actionable is missing. A human reading
   a small workflow can usually find it anyway, which is exactly why this is easy
   to leave out and why the agent case is the one that decides it.

   The pair is what "well enough that a model can correct itself" resolves to for
   this feature. Anything further - a suggested fix, a diff - is deliberately not
   claimed here.

.. feat_arch:: Validation splits into a validator and a defect
   :id: ARCH_WIRING
   :realises: FEAT_WIRING_REQUIRED_BOUND, FEAT_WIRING_BINDING_RESOLVES, FEAT_WIRING_TYPES_AGREE, FEAT_WIRING_ONE_DESIGNATED_OUTPUT, FEAT_WIRING_ALL_DEFECTS, FEAT_WIRING_DEFECT_LOCATED
   :uses: COMP_WIRING_VALIDATOR, COMP_WIRING_DEFECT
   :statement: Agconflo shall allocate workflow validation to the wiring validator and the wiring defect.

   Two components, each answerable for what the other cannot guarantee:

   - The wiring validator answers for all four defect classes and for
     ``FEAT_WIRING_ALL_DEFECTS``. Each class is a relation between a definition
     and the node types it names, and whether the walk continues after finding
     one is a property of the walk, which no value can promise about itself.
   - The wiring defect answers for ``FEAT_WIRING_DEFECT_LOCATED``. What a defect
     says about itself is true or false of one defect, independently of how many
     were found or of what found them.

   Splitting the defect from the validator is the division worth arguing. Folding
   them together would put "every defect is reported" and "a defect says where it
   is" on one subject, and they fail separately: a validator that stops at the
   first defect reports each of them perfectly, and one that finds all ten can
   describe every one as a type mismatch with no location. A component requirement
   needs one subject that owns its behaviour, and these are two behaviours.

   **A third component was drafted and dropped, and the reason is worth keeping.**
   The workflow definition looked like the natural owner of
   ``FEAT_WIRING_ONE_DESIGNATED_OUTPUT``: a signature is a property of one
   definition taken alone, so a definition could refuse to be assembled without
   exactly one output. That allocation contradicts ``FEAT_WIRING_ALL_DEFECTS``.
   A workflow with a malformed signature *and* three unbound parameters would
   report the signature alone, because assembly would fail before any wiring was
   examined - and the author would fix one defect, resubmit, and only then learn
   about the others. Two round trips is precisely what that requirement exists to
   prevent, so the signature is checked by the validator with everything else, and
   a definition carries whatever it was given.

   **The node types are not a component either, deliberately.** They are the other
   layer (``DEC_TWO_LAYERS``): a declaration this feature reads and checks a
   definition against, never something it defines or owns. No requirement here is
   about what a type declares - only about a definition disagreeing with one - so
   a component for them would own no behaviour. When loading a workflow lands, the
   catalogue that resolves a type name may well become one, with requirements of
   its own about a name that resolves to nothing.

   So the workflow definition and the node type declarations are this feature's
   inputs rather than its parts. They are data, and in Rust the type system is the
   detailed design of data - which is the same reason this project has no design
   level below the component requirement.

   The decisions this is built against are named here rather than linked:

   - ``DEC_TWO_LAYERS``: an instance is checked against its type, which is what
     gives the validator something to check against at all.
   - ``DEC_DECLARED_PARAMETERS``: the three lists a type declares are what
     "required", "optional" and "global" mean in a defect message.
   - ``DEC_BINDING_BY_PORT``: both ends of a binding are names in the definition,
     so every check here is possible without running anything.
   - ``DEC_WORKFLOW_SIGNATURE``: entry parameters and one designated output are
     what make a definition's shape declarable, and therefore checkable.
   - ``DEC_BACK_EDGES_ALLOWED``: a cycle is legal, so nothing here looks for a
     topological order, and reachability is asked from the entry nodes.
   - ``DEC_IMPLIED_CONTROL_EDGES`` and ``DEC_ROUTING_SEPARATE_FROM_CONTROL`` are
     recorded as an absence: the control graph they describe is derived rather
     than stored, and no requirement of this feature reads it. The validator
     examines bindings, which is the other graph.
