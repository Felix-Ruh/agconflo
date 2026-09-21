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
   definition never binds, a name that resolves to nothing, a name that resolves
   to more than one thing, a type that disagrees across a wire, and a signature
   that does not designate exactly one output.

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
   - **Several broken wires collapse into one defect.** Deduplicating by the name
     that failed to resolve is the tidy-looking version of this, and it leaves
     every wire but one unnamed. A defect is one thing wrong with one definition
     and carries one place, so five parameters bound to a deleted instance are
     five defects and five wires to repoint.

   Must pass unreported: a binding from an instance to itself, which is a cycle
   of length one and legal (``DEC_BACK_EDGES_ALLOWED``), and two instances of the
   same node type, which share a declaration and nothing else.

   Three shapes were left open here when this was first written: a binding naming
   a parameter the declaration does not carry, a designated output naming an
   instance that is not there, and two instances sharing one name. None of them is
   what this requirement's statement covers, which is a binding's source and an
   instance's type, so each is answered by a requirement of its own:
   ``CREQ_VALIDATOR_PARAMETER_DECLARED``, ``CREQ_VALIDATOR_OUTPUT_RESOLVES`` and
   ``CREQ_VALIDATOR_INSTANCE_NAMED_ONCE``. Answering them turned up a fourth,
   ``CREQ_VALIDATOR_PARAMETER_BOUND_ONCE``.

   Still open, and left so rather than guessed at:

   - **Two node types sharing one name within a definition.** The catalogue
     refuses the shape across documents (``CREQ_CATALOGUE_DECLARED_ONCE``), and a
     document cannot hold it, but a definition built by other means can, and the
     validator resolves the name to the first declaration carrying it.
   - **One parameter declared twice by one node type**, once as required and once
     as optional. A definition built by other means can hold it, and the validator
     reads the parameter as required.
   - **A binding into an entry node.** An entry node's parameters are the
     workflow's own (``DEC_WORKFLOW_SIGNATURE``), so a wire into one gives a
     parameter two sources, the caller and the wire - and a loop closing back onto
     an entry node draws exactly that (``DEC_BACK_EDGES_ALLOWED``). It is checked
     today like any other binding, for its source and its type.

   The first two are about the declarations a definition carries rather than
   about its wiring, which ``ARCH_WIRING`` keeps out of this feature, and the
   likely answer to both is a definition holding a catalogue rather than a list
   of declarations - a change to the model that the slice building definitions by
   other means, authoring, is the one to make. Until then the only thing asserted
   about them is that they do not stop the walk (``CREQ_VALIDATOR_EVERY_DEFECT``).

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
   - **A wire whose producer has no declaration is reported as a mismatch.** The
     producing instance names a node type the definition does not carry, so its
     output has no declared type at all; comparing against a stand-in for one -
     an empty name, a default - makes every wire out of that instance disagree.
     The missing type is already reported, and a second defect about the same
     wire sends the author to change a type that is not wrong.
   - **The check is skipped because something else about the wire was reported.**
     An instance with an unbound required parameter still declares types on the
     parameters that are bound, so a walk that moves on after an instance's first
     defect hides every disagreement below it. The author then fixes one defect
     and discovers the next, which is what reporting everything together exists
     to prevent.

   Must pass unreported: a binding whose output and parameter declare the same
   context type on two different node types, and one output bound by many
   parameters, which the model allows.

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

.. comp_req:: A binding to an undeclared parameter is a defect
   :id: CREQ_VALIDATOR_PARAMETER_DECLARED
   :derived_from: FEAT_WIRING_PARAMETER_DECLARED
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If a binding names a parameter that the node type of its instance does not declare, then Wiring validator shall report a defect naming that binding.

   The other end of a binding from the one ``CREQ_VALIDATOR_BINDING_RESOLVES``
   resolves. The parameter is looked up in the two lists a type declares
   parameters in, required and optional (``DEC_DECLARED_PARAMETERS``), and a name
   in neither is reported with the consuming instance and the parameter as it was
   written.

   Failure modes:

   - **The binding is passed over.** It fills no declared parameter, so a check
     starting from the declaration never visits it, and a type check needs a
     declared type it does not have. Measured: this is what the validator did, and
     a typo of an optional parameter passed with nothing reported.
   - **The lists are searched one short.** A binding to a declared optional
     parameter reported as undeclared refuses a correct workflow, and makes the
     second list meaningless.
   - **A requested global counts as a parameter.** Globals are read by declaration
     and never wired, so a binding spelled like one fills nothing.
   - **An instance of a node type that was not supplied is reported.** Its
     declaration is what is missing, so whether it declares a parameter is
     unknowable, and the missing type is already in the report.
   - **Its source goes unchecked.** An undeclared parameter bound to an instance
     that is not there is two defects and two fixes, the parameter renamed and the
     source repointed, and moving on once the parameter is found wanting leaves
     the second for the next round trip. Its type is a different matter: an
     undeclared parameter has no declared type to compare, so none is compared.

   Must pass unreported: a binding to an optional parameter, on an instance whose
   required parameters are all bound as well.

.. comp_req:: An output naming no instance is a defect
   :id: CREQ_VALIDATOR_OUTPUT_RESOLVES
   :derived_from: FEAT_WIRING_OUTPUT_RESOLVES
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If the one output a definition designates names no instance of it, then Wiring validator shall report a defect naming that definition and the name it designates.

   The designation is resolved against the definition's instances as a binding's
   source is. One naming nothing is reported with the definition as its place,
   since a signature defect concerns the definition and no node in it
   (``CREQ_DEFECT_NAMES_PLACE``), and with the name as it was written.

   Only a single designation is resolved. A definition designating several is
   refused for that already (``CREQ_VALIDATOR_ONE_OUTPUT``), which of them was
   meant is what its author has to decide first, and resolving each of them as
   well would name the definition once per designation - several defects about one
   place. A document cannot hold more than one in any case (``DEC_ONE_OUTPUT_KEY``).

   Failure modes:

   - **The designation is counted and never resolved.** Measured: a definition
     designating one name that no instance carries passed with nothing reported,
     so a workflow whose result will never be produced was accepted.
   - **It is reported as designating nothing.** The defect for designating none
     carries a count, and a count of nought loses the name the author typed,
     which is exactly what has to be echoed back.
   - **It is reported beside the count.** A definition designating two names, one
     of them unresolved, then carries two defects about its outputs, where
     choosing one answers both.
   - **A shared name is reported as naming nothing.** It names more than one
     instance, which is a defect of its own
     (``CREQ_VALIDATOR_INSTANCE_NAMED_ONCE``), and reporting it here as well sends
     the author to create an instance that already exists twice.

   Must pass unreported: an output naming an entry node, and one naming an
   instance whose name is not its node type's.

.. comp_req:: A name several instances share is a defect
   :id: CREQ_VALIDATOR_INSTANCE_NAMED_ONCE
   :derived_from: FEAT_WIRING_INSTANCE_NAMED_ONCE
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If several instances of a definition share one name, then Wiring validator shall report one defect naming that name.

   A name that reaches two instances is not a place, so nothing about either of
   them can be said through it. The name is reported once, where it first
   appears, and no instance carrying it is looked at further - not its type, not
   its bindings, and not the context type of a wire from it. The author untangles
   the name and learns what is behind it from the next report; the alternative is
   a report in which every line about those instances names a node the author
   cannot find. A binding or an output naming it does resolve, to more than one
   thing, which is this defect and not an unresolved one.

   Failure modes:

   - **The name resolves to the first instance carrying it.** What a lookup does
     by default, and measured here: a wire from the shared name was compared with
     the first instance's output type, so the verdict turned on which was written
     first.
   - **Each instance is checked as if the name were its own.** Measured: two
     instances of one type, both unbound, reported one defect twice word for word,
     and two wired to different missing sources reported two defects at one place
     - both of them what ``CREQ_VALIDATOR_EVERY_DEFECT`` rules out.
   - **The name is reported once per instance.** Three instances sharing it are
     one name to change, not three defects.
   - **A wire or an output naming it is reported as naming nothing.** It names
     more than one thing, and sending the author to create it is wrong twice over.
   - **Names of different kinds are compared.** An instance named like its node
     type, or like a parameter, shares a spelling and nothing else. Instance names
     are compared with instance names.

   Must pass unreported: two instances of one node type under different names, and
   an instance named like its own node type or like a parameter.

.. comp_req:: A parameter bound twice is a defect
   :id: CREQ_VALIDATOR_PARAMETER_BOUND_ONCE
   :derived_from: FEAT_WIRING_PARAMETER_BOUND_ONCE
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If an instance binds one parameter more than once, then Wiring validator shall report one defect naming that instance and parameter.

   A parameter binds to exactly one output (``DEC_BINDING_BY_PORT``), so two
   bindings for it are two answers where one is wanted. It is reported once, at
   its first binding, and none of its bindings is checked for its source or its
   type: each of those would be a check of a wire the author may be about to
   delete. Whether the parameter is declared is still checked, once, because
   that does not depend on which binding is kept.

   Failure modes:

   - **Each binding is checked on its own.** Measured, in three forms: two
     bindings to one missing source reported one defect twice, two to different
     missing sources reported two defects at one place, and two to sound sources
     reported nothing at all - a node with two inputs where its type declares one.
   - **The first binding is checked and the rest are ignored.** It chooses on the
     author's behalf which wire is real.
   - **It is reported once per binding beyond the first.** A parameter bound three
     times is one parameter to fix.
   - **One parameter name on two instances is reported.** Each instance has
     parameters of its own, and two instances each binding ``input`` is the
     ordinary shape of a graph.

   Must pass unreported: one parameter name bound once on each of several
   instances, and one output bound by two parameters of one instance.

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

   Must pass: a definition with a defect of each of the four classes first
   written reports all of them, each once; and so does one carrying each of the
   four classes about names beside an unbound parameter, on the same instance as
   two of them.

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

.. comp_req:: A defect carries its place as a value
   :id: CREQ_DEFECT_NAMES_PLACE
   :derived_from: FEAT_WIRING_DEFECT_LOCATED
   :allocated_to: COMP_WIRING_DEFECT
   :ears_pattern: ubiquitous
   :statement: Wiring defect shall carry the place it concerns as a value rather than only in its message.

   What a defect says about itself, independently of how many were found, and
   what this adds to its parent: the parent asks that the place be named, and a
   validator that names it in well-written prose satisfies that. This asks that
   the place be readable without reading the prose.

   The place is the node instance and the parameter for a defect about a wire -
   a parameter unbound, bound twice or undeclared, or a binding that resolves to
   nothing or disagrees on its type; the instance alone for a defect about an
   instance, which is one of a node type the definition does not carry and whose
   parameter list is therefore unknowable, or a name several instances share, the
   one place such a defect can name; and the definition itself for a signature
   defect, an output naming no instance among them. That is the distinction the
   parent leaves to this level, because the classes of defect are here.

   Failure modes:

   - **The place is only in the message.** A caller then parses prose to find the
     node, and an agent correcting its own workflow is the caller this feature
     was written for.
   - **A signature defect invents a parameter.** It concerns the definition, and
     naming a node to fill the field would send the author to the wrong place.
   - **A defect about an unknown name reports nothing.** The name the author
     typed is exactly what has to be echoed back, even though it resolves to
     nothing - and especially then.
