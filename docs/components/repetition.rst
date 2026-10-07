=========================================
Components of a workflow repeating itself
=========================================

The requirements ``ARCH_REPETITION`` allocates to the components it uses,
each defined where its first feature put it: the topology reader in
``components/topology``, the wiring validator in ``components/wiring``, and
the run scheduler and the workflow run in ``components/run``. Each title is
the grammatical subject of
the requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

An instance's passes follow from its wiring and the first contexts its
bindings declare (``DEC_PASS_CLOCKS``): the run's one pass, a router's passes
on which it takes a branch naming the instance, a declared first context
followed by the passes of what the binding carries, or a cycle's own passes.
One set of
passes encloses another when each pass of the other is part of one of its
own. An input on the instance's own passes is taken pass by pass; one on
passes enclosing them is read from the pass the activation belongs to.

.. comp_req:: An activation is given each input of its own pass
   :id: CREQ_SCHEDULER_GIVES_ONE_PASS
   :derived_from: FEAT_REPEAT_ON_NEW_CONTEXTS
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: event
   :statement: When an instance reads an output made on its own passes, Run scheduler shall give each of its activations the context made on the same pass.

   An input a router passes on is made on the pass of the router on which it
   takes a branch naming the instance; an output on the pass of the instance
   that made it.

   Failure modes:

   - **The earliest context not yet taken given**, and a node joining a
     branch taken on some passes with one taken on all given two passes'
     contexts (``EVD_PASSES_MISPAIRED_ACROSS_A_BRANCH``), which is what the
     scheduler did.
   - **The latest context given**, and what a pass is given turning on the
     order instances are offered in (``EVD_STANDING_BY_INSTANCE_ORDER``).
   - **A context given twice**, and a node answering the same pass again.

.. comp_req:: An instance is offered on its next pass once its inputs hold it
   :id: CREQ_SCHEDULER_OFFERS_AGAIN
   :derived_from: FEAT_REPEAT_ON_NEW_CONTEXTS
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: event
   :statement: When every input of an instance holds a context of the next of its passes, Run scheduler shall offer that instance for activation.

   An instance on the run's one pass is offered once, and one on no pass
   never. Restated by ``DEC_CHANGE_SCHEDULER_OFFERS_AGAIN``.

   Failure modes:

   - **An instance offered once and never again**, and a repetition that
     stops after its first pass.
   - **An instance offered again on inputs of a pass it has run**, and a
     node running for ever on the same contexts until the budget stops it.
   - **An instance offered before every input holds its next pass**, and
     given another pass's context or none.

.. comp_req:: An activation reads what was made on enclosing passes from the pass it belongs to
   :id: CREQ_SCHEDULER_READS_ENCLOSING_PASS
   :derived_from: FEAT_ENCLOSING_PASS_SERVES
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: event
   :statement: When an instance reads an output made on passes enclosing its own, Run scheduler shall give each of its activations the context made on the enclosing pass that activation belongs to.

   A pass of a router's branch belongs to the router's pass on which it took
   the branch; every pass belongs to the run's one pass. A pass on a set of a
   router's branches belongs, among the router's passes on a larger set, to
   the one it was taken on.

   Failure modes:

   - **A context made once taken once**, and a repetition reading the brief
     stalling on its second pass.
   - **A branch's node given the enclosing pass's first or latest context**,
     not the one of the pass the branch was taken on.

.. comp_req:: Instances whose inputs share no pass are found
   :id: CREQ_SCHEDULER_REPORTS_UNPAIRED
   :derived_from: FEAT_PASSES_PAIRED_BEFORE_RUN
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: unwanted
   :statement: If the inputs of an instance share no pass, then Run scheduler shall report that instance with the passes each of its inputs comes on.

   Passes are worked out from the wiring and the first contexts its bindings
   declare, before anything runs (``DEC_PASS_CLOCKS``), and the wiring
   validator reports what this finds (``CREQ_VALIDATOR_REPORTS_UNPAIRED``).
   Inputs share a pass when one's passes are enclosed by all the others', or
   when they come from branches of one router a single branch names together.
   An instance on a cycle no first context starts runs on no pass and is not
   reported here: the validator reports the cycle
   (``CREQ_VALIDATOR_CYCLE_STARTED``).

   Failure modes:

   - **Two branches no route takes together passed**, and their join waiting
     for ever.
   - **A loop's node reading a branch of the loop passed**, and given another
     pass's context or waiting for ever.
   - **A join of two branches a route takes together reported**, refusing a
     workflow whose node has a pass.
   - **A cycle nothing starts reported as unpaired**, beside its own
     defect, and the one fault named twice in two ways.

.. comp_req:: A cycle no first context starts is a defect
   :id: CREQ_VALIDATOR_CYCLE_STARTED
   :derived_from: FEAT_PASSES_PAIRED_BEFORE_RUN
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If instances of a workflow reach one another through bindings none of which declares its first context, then Wiring validator shall report a defect naming those instances.

   ``DEC_CYCLE_STARTED_BY_A_FIRST``: every instance on such a cycle waits for
   another on it, so none can be given contexts of any pass. The cycle is
   reported once, its instances in the definition's order, and an instance
   reading from it is not reported beside it. Asked of a definition carrying
   no other defect, whose bindings all resolve.

   Failure modes:

   - **The cycle passed**, and part of a run done before it stops with no
     node able to go on (``EVD_LOOP_FIRST_CONTEXT_UNCHECKED``).
   - **A cycle passed because one binding on it declares a first context**,
     where another edge back declares none and closes a cycle of its own.
   - **Each instance on it reported**, or every instance reading from it,
     and the one forgotten first context named many times.
   - **A cycle a first context starts reported**, refusing a loop that runs.

.. comp_req:: An instance whose inputs share no pass is a defect
   :id: CREQ_VALIDATOR_REPORTS_UNPAIRED
   :derived_from: FEAT_PASSES_PAIRED_BEFORE_RUN
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If the inputs of an instance of a workflow share no pass, then Wiring validator shall report a defect naming that instance and the passes each of its inputs comes on.

   ``DEC_PAIRING_IS_WIRING``: the passes follow from the definition alone, so
   the check is the validator's, and ``agconflo check`` reports it. Asked of
   a definition carrying no other defect and no cycle nothing starts.

   Failure modes:

   - **The instance found only when a run starts**, and a check passing a
     workflow every run of it refuses (``EVD_LOOP_FIRST_CONTEXT_UNCHECKED``).
   - **Passes worked out from bindings that do not resolve**, and defects
     beside a broken binding that are its echo.

.. comp_req:: A run whose nodes cannot be given one pass's contexts is refused
   :id: CREQ_RUN_REFUSES_UNPAIRED
   :derived_from: FEAT_PASSES_PAIRED_BEFORE_RUN
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If an instance of a workflow cannot be given contexts of one pass, then Workflow run shall refuse to start naming every such instance.

   An instance whose inputs share no pass is a defect of the wiring
   (``DEC_PAIRING_IS_WIRING``), reported by the wiring validator
   (``CREQ_VALIDATOR_REPORTS_UNPAIRED``), so the run refuses it as it refuses
   every wiring defect, before its signature is asked
   (``CREQ_RUN_REFUSES_DEFECTS``), naming each such instance with the passes
   of its inputs.

   Failure modes:

   - **The run started**, and a node's activation given two passes' contexts
     or none after half of an expensive run.
   - **The first such instance named and the rest left**, and a fix found one
     instance at a time.

.. comp_req:: An output is given to every instance reading it on the pass it was made
   :id: CREQ_RUN_WALKS_EVERY_EDGE
   :derived_from: FEAT_REPEAT_ON_NEW_CONTEXTS
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When the caller reports the output of an instance whose node type does not route, Workflow run shall hold it as the output of the instance's next pass for every instance reading it.

   Restated by ``DEC_CHANGE_RUN_WALKS_EVERY_EDGE``.

   Failure modes:

   - **The output replacing an earlier pass's**, and a reader that has not
     yet run that pass given the later one.
   - **The output held for one reader of several**, and another waiting for
     a pass that went elsewhere.

.. comp_req:: A binding's declared first context is given on its first pass
   :id: CREQ_SCHEDULER_GIVES_FIRST
   :derived_from: FEAT_FIRST_CONTEXT_DECLARED
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: event
   :statement: When a binding declares its first context, Run scheduler shall give that context to the binding's instance on the binding's first pass and what the binding carries on each pass after.

   ``DEC_FIRST_CONTEXT_DECLARED``: it is the binding's pass 0, and the passes
   of what the binding carries are its passes 1 onward, whenever they are made
   (``DEC_PASS_CLOCKS``).

   Failure modes:

   - **The context placed after what was walked first**, and what a pass is
     given turned on whether the binding's source happened to run before the
     instance.
   - **The context given for every pass**, as a parameter nothing binds is,
     and the wire's contexts never taken.
   - **The context given to another binding of the instance**, or to the
     same parameter of another instance.

.. comp_req:: A run holds each first context its workflow declares
   :id: CREQ_RUN_HOLDS_DECLARED_FIRST
   :derived_from: FEAT_FIRST_CONTEXT_DECLARED
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When a run starts, Workflow run shall hold for each binding declaring its first context a context holding the declared text, of the type the binding's parameter is declared for.

   ``DEC_FIRST_CONTEXT_DECLARED``: made when the run starts, after its wiring
   and its signature are checked, from the identifier source its caller lends
   it, and held as an argument is, so that its identifier is checked against
   the arguments' (``CREQ_RUN_REFUSES_SHARED_ARGUMENT_IDENTIFIER``). A source
   that has issued every identifier refuses the run.

   Failure modes:

   - **The context made from a source of its own**, and two contexts of the
     run under one identifier.
   - **Its type taken from elsewhere**, the source's output type or a
     made-up one, and a node given a context its parameter is not declared
     for.
   - **A context made for every pass**, and the run's contexts growing with
     nothing to say it.
   - **An exhausted source passed over**, and a context with no identifier
     of its own.

.. comp_req:: A binding's first context is read as its text
   :id: CREQ_READER_READS_FIRST
   :derived_from: FEAT_FIRST_CONTEXT_DECLARED
   :allocated_to: COMP_TOPOLOGY_READER
   :ears_pattern: event
   :statement: When a binding is written as a table holding first, Topology reader shall read the text first holds as the first context the binding declares.

   ``DEC_FIRST_CONTEXT_DECLARED``: ``first`` is a table holding ``text``, a
   string, and nothing else, in a binding table beside ``from`` and, for a
   router's input, ``input``. A binding table declaring no ``first`` still
   needs its ``input`` (``CREQ_READER_READS_ROUTED_INPUT``). A value of
   another kind is refused where it is written
   (``CREQ_READER_FAULT_LOCATED``).

   Failure modes:

   - **The key read past**, and a loop whose first context is declared
     never started.
   - **The text changed on the way**, trimmed or its line breaks dropped,
     and a node given another first context than the one written.
   - **A malformed first context read as none**, and a mistake in it found
     only as a loop that never starts.
   - **The input left optional for every binding table**, and a binding
     meant to take a router's input read as taking its output.
