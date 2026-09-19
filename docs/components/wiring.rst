==============================
Components of graph validation
==============================

The two parts ``ARCH_WIRING`` divides validating a workflow into, and the
requirements allocated to each. A component is an object rather than a level:
nothing derives from it, and it exists so that a requirement has one subject
answerable for it - which is also what lets a requirement's grammatical subject
be checked against the title of the component it is allocated to.

Each title below is that subject. "Workflow definition shall ..." reads as a
sentence about one thing, and the gate in ``scripts/gates`` refuses a component
requirement whose subject is anything else.

.. comp:: Wiring validator
   :id: COMP_WIRING_VALIDATOR
   :crate: agconflo-core

   The walk that compares one workflow definition against the node types it names,
   and collects what it finds. Every defect class belongs here: a parameter the
   definition never binds, a name that resolves to nothing, a type that disagrees
   across a wire, and a signature that does not designate exactly one output.

   The definition and the type declarations are its inputs rather than parts of
   this feature - data it reads, and the other layer it reads them against
   (``DEC_TWO_LAYERS``).

   It answers for the completeness of its own report. Continuing after the first
   defect is a property of the walk and of nothing else, which is why it is
   allocated here rather than to what the walk produces.

.. comp:: Wiring defect
   :id: COMP_WIRING_DEFECT
   :crate: agconflo-core

   One thing wrong with one definition: what is wrong, and where. It is a value
   rather than a message, so that a caller can act on it - an agent correcting its
   own workflow reads the location, not the prose.

   Its requirements are true or false of a single defect taken alone, which is
   what separates it from the validator that produced it.

.. comp_req:: A required parameter with no binding is a defect
   :id: CREQ_VALIDATOR_REQUIRED_BOUND
   :derived_from: FEAT_WIRING_REQUIRED_BOUND
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If a node instance leaves a required parameter unbound, then Wiring validator shall report a defect naming that parameter.

   The declaration says what the node cannot run without
   (``DEC_DECLARED_PARAMETERS``), and this compares the bindings a definition
   carries against it. The comparison runs over the declaration rather than over
   the bindings, which is the whole of the difference between finding this defect
   and never seeing it.

   Failure modes:

   - **The walk iterates over the bindings.** Every binding present is then
     checked and a parameter with none is never visited, so a workflow missing
     half its wires passes. This is the defect this requirement exists to rule
     out, and it is invisible to every other requirement here.
   - **An optional parameter with no binding is reported.** Over-blocking, and it
     makes the second of the two declared lists meaningless.
   - **A declared global type is treated as a parameter.** Globals are read by
     declaration rather than wired, so demanding a binding for one refuses a
     workflow that is correct.

   Must pass unreported: a node whose optional parameters are all unbound, a node
   with no parameters at all, and an entry node, whose parameters are the
   workflow's own.

.. comp_req:: A binding that names nothing is a defect
   :id: CREQ_VALIDATOR_BINDING_RESOLVES
   :derived_from: FEAT_WIRING_BINDING_RESOLVES
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If a binding names an instance or a node type that the definition does not carry, then Wiring validator shall report a defect naming that binding.

   Both ends of a binding are names (``DEC_BINDING_BY_PORT``), and a name that
   resolves to nothing is a wire to nowhere. The instance is one layer and its
   type is the other (``DEC_TWO_LAYERS``), so both resolutions belong here.

   Failure modes:

   - **A renamed or deleted instance leaves a binding behind.** The parameter is
     bound, so the requirement above is satisfied, and the wire points at
     nothing.
   - **An instance names a node type that was not supplied.** Its parameters are
     then unknowable, so every other check on that instance is vacuous rather
     than passing - which is the reason this is reported rather than skipped.
   - **The defect is reported once per consumer rather than once per name.** The
     report then grows with the graph instead of with the mistake.

   Must pass unreported: a binding from an instance to itself, which is a cycle
   of length one and legal (``DEC_BACK_EDGES_ALLOWED``), and two instances of the
   same node type, which share a declaration and nothing else.

.. comp_req:: A binding across two context types is a defect
   :id: CREQ_VALIDATOR_TYPES_AGREE
   :derived_from: FEAT_WIRING_TYPES_AGREE
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If a binding joins an output to a parameter declared for another context type, then Wiring validator shall report a defect naming that binding.

   Nominal typing earns its place here: the declared type of the producing node's
   output is compared with the declared type of the consuming parameter, and a
   disagreement is refused before a node is ever handed the wrong thing.

   Failure modes:

   - **The comparison is loosened.** Matching case-insensitively, by prefix, or
     after trimming makes two distinct types compare as one, and the node that
     receives the wrong context produces a confident wrong answer rather than
     failing.
   - **An unknown type name is treated as agreeing.** A name nothing declares
     would then silently satisfy every binding it appears in.
   - **The check is skipped when either end is already defective.** The author
     then fixes one defect and discovers the next, which is what reporting
     everything together exists to prevent.

   Must pass unreported: a binding between two parameters of the same declared
   type on different node types, and one output bound to many parameters, which
   the model allows.

.. comp_req:: A signature without exactly one output is a defect
   :id: CREQ_VALIDATOR_ONE_OUTPUT
   :derived_from: FEAT_WIRING_ONE_DESIGNATED_OUTPUT
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If a definition designates no output or designates more than one, then Wiring validator shall report a defect naming that definition.

   A workflow is wired into another workflow through its signature
   (``DEC_WORKFLOW_SIGNATURE``), so a definition that declares no result, or
   several, cannot be composed and is malformed on its own terms.

   Failure modes:

   - **The check refuses the definition before the wiring is examined.** The
     author is then told about the signature and nothing else, and learns the
     rest only after fixing it. This is why the check lives here rather than
     where a definition is assembled.
   - **A terminal node is picked when none is designated.** The workflow then has
     a result decided by the engine, which changes silently when a node is added.

   Must pass unreported: a workflow with no entry parameters, which is legal, and
   a designated output whose node also feeds other nodes, which does not make it
   less terminal.

.. comp_req:: Every defect is found in one pass
   :id: CREQ_VALIDATOR_EVERY_DEFECT
   :derived_from: FEAT_WIRING_ALL_DEFECTS
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: ubiquitous
   :statement: Wiring validator shall report every defect of a definition rather than stopping at the first.

   A property of the walk rather than of what it finds, which is why it is
   allocated here and not to the defect.

   Failure modes:

   - **The walk returns on the first defect.** No wording improves this; it is a
     property of the control flow, and it turns ten defects into ten round trips
     for an author that pays per round trip.
   - **A defect is reported twice.** Once per direction of a binding, or once per
     check that touches it, so a report grows without saying more.
   - **A malformed definition stops the walk another way.** An index out of range
     or an unwrap on a missing type ends the run with a panic rather than a
     report, which is the same failure wearing different clothes.

   Must pass: a definition with defects of all four classes at once reports all
   of them, each once.

.. comp_req:: A workflow without defects is accepted
   :id: CREQ_VALIDATOR_ACCEPTS_WELL_FORMED
   :derived_from: FEAT_WIRING_ACCEPTS_WELL_FORMED
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: ubiquitous
   :statement: Wiring validator shall report nothing for a definition that carries none of the defects it checks for.

   The control on everything above. Each requirement here says what must be
   refused and none says what must not be, so a validator that reports a defect
   for every definition satisfies all of them.

   Failure modes:

   - **A legal shape is refused.** A cycle, a node nothing reaches, an unbound
     optional parameter, a declared global with no wire, an output bound by
     several parameters: each is well formed, and each is what a validator
     written to be strict refuses by accident.
   - **The empty definition is refused or accepted for the wrong reason.** A
     workflow with no instances has no wiring defect and no designated output, so
     it is refused for its signature alone - which is worth pinning, because it
     is the case where two requirements could quietly both fire.

.. comp_req:: A defect says where it is
   :id: CREQ_DEFECT_NAMES_PLACE
   :derived_from: FEAT_WIRING_DEFECT_LOCATED
   :allocated_to: COMP_WIRING_DEFECT
   :ears_pattern: ubiquitous
   :statement: Wiring defect shall name the place in the definition that it concerns.

   What a defect says about itself, independently of how many were found. The
   place is the node instance and the parameter for a binding defect, and the
   definition itself for a signature defect - so "where" is part of the value
   rather than a sentence about it.

   Failure modes:

   - **The place is only in the message.** A caller then parses prose to find the
     node, and an agent correcting its own workflow is the caller this feature
     was written for.
   - **A signature defect invents a parameter.** It concerns the definition, and
     naming a node to fill the field would send the author to the wrong place.
   - **A defect about an unknown name reports nothing.** The name the author
     typed is exactly what has to be echoed back, even though it resolves to
     nothing - and especially then.
