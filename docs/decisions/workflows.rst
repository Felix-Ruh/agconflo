=====================================
Decisions about development workflows
=====================================

How the development workflows Agconflo runs on itself are built
(``STKH_SELF_HOSTING``). The first records stakeholder goals the maintainer
has approved, in ``workflows/record-goals``. These decisions were measured on
it (``evidence/workflows``); the ones not bound to that task are meant for the
workflows that follow.

A workflow here is the structure a weak model works inside: which question
each step asks, what a script checks, where a step is sent back to. The
decisions below are about that structure. The engine's own decisions are
elsewhere, and none of these changes them.

.. dec:: Development workflows are measured on a weak model
   :id: DEC_WORKFLOWS_ON_A_WEAK_MODEL
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_JUDGE_WEIGHS_WHAT_WAS_RAISED
   :statement: Agconflo's development workflows shall be built and measured with deepseek-v4.1-flash for every model step, the judge's included.

   The maintainer's decision: a workflow is there to make a weak model do
   what it could not do in one call, so its worth is measured with one. A
   stronger model in any step hides whether the structure did the work.
   Versions 5 to 7 judged with ``deepseek/deepseek-v4-pro-0813``, and version
   7 was then accepted on its first pass with defects no step had raised
   (``EVD_JUDGE_WEIGHS_WHAT_WAS_RAISED``): the stronger judge gained nothing
   the review had not given it. A stronger model is for when a workflow is
   used for real work, not while it is being built.

.. dec:: A development workflow serves one kind of pull request, any of that kind
   :id: DEC_WORKFLOW_PER_KIND_OF_CHANGE
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_RECORD_GOALS_ON_TWO_PULL_REQUESTS
   :statement: Each development workflow shall serve one kind of pull request and be run unchanged on at least two past pull requests of that kind before it is relied on.

   A workflow fitted to one pull request learns its answer: versions 3 to 7
   had its two goals, its file and its topics written into them. The
   record-goals workflow takes any number of goals from its brief and
   chooses a file for each, and it was checked on #55 and #29, which differ
   in both (``EVD_RECORD_GOALS_ON_TWO_PULL_REQUESTS``). Each past pull
   request is run from its parent commit, and what it merged is the answer
   the run is compared with. Other kinds of change, such as code or a
   decision with its evidence, get workflows of their own.

.. dec:: Each model step asks one question, in a form a script checks
   :id: DEC_ONE_QUESTION_PER_MODEL_STEP
   :dec_status: accepted
   :decided_on: 2026-09-27
   :statement: Each model step of a development workflow shall ask one question and take its answer in a fixed form that a script checks before the answer is used.

   A weak model given several things to decide at once answered some of
   them: version 4's step deciding seven questions first answered three. One
   question per step makes the form small enough to check - one line naming
   a file, one naming an id, four lines naming a goal - and a check that
   fails says exactly what to correct. Where the answer must come from what
   the step was shown, the check holds it to that: an id must be one the
   graph returned, and a sentence said to be quoted must be found where it
   was quoted from.

.. dec:: File and command work is done by scripts and wired tool steps
   :id: DEC_NO_TOOLS_FOR_MODEL_STEPS
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_WEAK_MODEL_WRITES_CALL_MARKUP, EVD_NO_TOOLS_NO_MARKUP
   :statement: A development workflow shall read, write and run through scripts and wired tool steps rather than through tools offered to a model.

   Offered tools, the weak model called one that was not offered, and the
   run failed; offered none, it wrote its calls out as its answer
   (``EVD_WEAK_MODEL_WRITES_CALL_MARKUP``). With the work moved into command
   steps whose command a script writes, and a script splicing a model's
   answer into the file, neither happened again in 406 answers
   (``EVD_NO_TOOLS_NO_MARKUP``). A model then answers questions and never
   acts; what it may change is what a script lets its answer change.

.. dec:: Every prompt names its tools or says it has none
   :id: DEC_PROMPT_NAMES_ITS_TOOLS
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_WEAK_MODEL_WRITES_CALL_MARKUP, EVD_NO_TOOLS_NO_MARKUP
   :statement: Every prompt of a development workflow shall name each tool its step offers and say that no other exists, or say that the step offers none.

   The tools a step offers already reach the model as function definitions,
   well formed (``EVD_NO_TOOLS_NO_MARKUP``), and the weak model still reached
   for a shell it was not offered. The prompt says it again in words, with
   each parameter, because the definitions alone did not stop it.

.. dec:: A step inside a loop checks its own answer and asks again
   :id: DEC_RETRY_INSIDE_A_LOOPED_STEP
   :dec_status: accepted
   :decided_on: 2026-09-27
   :statement: A model step inside a loop shall check its own answer and ask again with the reasons within its activation, at most three times, rather than being sent back by a router of its own.

   A step sent back by its own validator and by the loop's judge would take
   its goal on the passes the judge sends back on and its feedback on those
   its validator does, and no one of them encloses the other. No activation
   of it could be given one pass's contexts (``DEC_PASS_CLOCKS``), and
   ``agconflo check`` refuses such a workflow before anything runs
   (``DEC_PAIRING_IS_WIRING``). So a looped step keeps its retries inside its
   activation, where the record still holds each attempt as an exchange.
   Outside a loop a validator router sending its step back is accepted.
   Versions 4 to 7 used that, with a holder standing after it, which a node
   type can no longer declare (``DEC_STANDING_REFUSED``).

.. dec:: What the judge can fault is inside the loop it sends back into
   :id: DEC_JUDGED_STEPS_IN_THE_LOOP
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_JUDGED_STEP_OUTSIDE_THE_LOOP
   :statement: Every step whose output a development workflow's judge can find at fault shall run inside the loop the judge sends back into.

   A defect in a step that ran once before the loop is found again on every
   pass and never fixed (``EVD_JUDGED_STEP_OUTSIDE_THE_LOOP``). In the
   record-goals workflow every step from choosing the file to writing it
   runs on each attempt, and each sees the judge's feedback in the goal it is
   given.

.. dec:: The record-goals workflow takes its goals one at a time
   :id: DEC_ONE_GOAL_PER_PASS
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_RECORD_GOALS_ON_TWO_PULL_REQUESTS, EVD_JUDGE_STATE_PASSED_ON_ALIKE
   :statement: The record-goals workflow shall record its brief's goals one at a time in a single loop whose judge's output is the loop's state.

   A workflow's instances are fixed when it is written, and a brief may hold
   any number of goals. One loop over them keeps the graph the same size for
   one goal or ten. The judge's output says which goal and which pass comes
   next and carries the feedback; the step heading the loop reads nothing
   else. Each accepted goal is staged in git's index, and each attempt starts
   from the index, so a rejected attempt leaves nothing behind and the
   accepted ones are on disk for the next goal to be placed beside.

   The judge was the loop's router until routers stopped making contexts
   (``DEC_ROUTER_PASSES_ON_ITS_INPUTS``). It is now a step like any other,
   and a router after it passes its output back round or on, which changed
   nothing any step was given (``EVD_JUDGE_STATE_PASSED_ON_ALIKE``).

.. dec:: The review asks one kind of finding per step
   :id: DEC_REVIEW_ONE_KIND_PER_STEP
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_JUDGE_WEIGHS_WHAT_WAS_RAISED
   :statement: A development workflow's review shall ask for each kind of finding in a step of its own.

   The judge can weigh only what the review raises
   (``EVD_JUDGE_WEIGHS_WHAT_WAS_RAISED``), and one open question - write 8 to
   12 hypotheses about the change - left the weak model to choose what to
   look at. The record-goals review has four: closing words, found by a
   script; claims about how Agconflo behaves, each with the need that
   supports it; each sentence naming a need, which a script pairs with that
   need's statement from the graph for the model to mark correct or wrong;
   and each sentence of the maintainer's note, marked kept, missing or
   contradicted.

.. dec:: A defect rests on a rule the judge quotes
   :id: DEC_JUDGE_QUOTES_ITS_RULE
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_JUDGE_WITHOUT_A_RULE_TO_QUOTE, EVD_JUDGE_WITH_A_POLICY
   :statement: A development workflow's judge shall count a finding as a defect only when it quotes, word for word, the sentence of the task, the project's rules or the workflow's review policy that the finding breaks.

   The judge fills in a form per finding: what was found, the rule, the
   sentence of the diff, and whether it is a defect. A script finds the
   quoted rule where it was quoted from, so a defect cannot rest on a rule
   the model made up. The rules must then hold a sentence for each kind of
   finding the review raises: without one for claims and relations the weak
   judge upheld nothing (``EVD_JUDGE_WITHOUT_A_RULE_TO_QUOTE``), and with a
   review policy sentence for each it sent goals back for real defects
   (``EVD_JUDGE_WITH_A_POLICY``).

.. dec:: The workflow's documents are written by its generator
   :id: DEC_WORKFLOW_DOCUMENTS_GENERATED
   :dec_status: superseded
   :decided_on: 2026-09-27
   :statement: The record-goals workflow's documents shall be changed by changing its generator and committed as the generator writes them.

   Its scripts share their helpers - the checks, the self-check loop, the
   tool preamble - and Lua scripts here cannot load one another, so each
   carries its own copy. Written by hand, a fix would have to reach every
   copy. The generated documents are committed, so the workflow runs with
   ``sh``, Docker and ``agconflo`` alone; Python is needed only to change it.
   ``DEC_NO_PYTHON`` is about building the requirements project, which this
   does not touch.

   Superseded by ``DEC_WORKFLOW_SCRIPTS_SHARE_A_MODULE``: versions 1 to 10
   were written by this generator.

.. dec:: The workflow's scripts share their helpers through a module
   :id: DEC_WORKFLOW_SCRIPTS_SHARE_A_MODULE
   :dec_status: accepted
   :decided_on: 2026-10-07
   :supersedes: DEC_WORKFLOW_DOCUMENTS_GENERATED
   :supported_by: EVD_SCRIPTS_REWRITTEN_ALIKE
   :statement: The record-goals workflow's documents shall be written by hand, the helpers its scripts share kept once in a module its scripts require.

   The generator stood in for what scripts could not do: load one another.
   A manifest now names modules its scripts require by name
   (``DEC_MODULES_REQUIRED_BY_NAME``), and the repository holds no Python
   (``DEC_NO_PYTHON_IN_THE_REPOSITORY``), so the generator's one reason is
   gone and its language is ruled out. Its run tracer goes with it: reading a
   run's record is the planned viewer's, not a workflow's.

   ``lib/help.lua`` holds the checks, the step that asks a model and asks
   again with the reasons, and the text every prompt shares - the tool
   preamble and the note on the goal's feedback - so a change to any of them
   reaches every step at once. The scripts were rewritten from what the
   generator wrote, and ran as it did, prompt for prompt
   (``EVD_SCRIPTS_REWRITTEN_ALIKE``).

.. dec:: The brief gives each goal's stakeholder
   :id: DEC_STAKEHOLDER_IN_THE_BRIEF
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_STAKEHOLDER_FROM_THE_DEFINITION, EVD_STAKEHOLDER_FROM_THE_BRIEF
   :statement: The record-goals workflow shall record each goal with the stakeholder its brief gives, and choose one by precedent only for a goal the brief gives none.

   Whose goal it is belongs to the maintainer who approves it, with the
   statement: #29's commit says its goals were "worded with the
   stakeholder". Chosen by the weak model from the definition it matched
   once in five, and by precedent three times in five
   (``EVD_STAKEHOLDER_FROM_THE_DEFINITION``); given, five in five
   (``EVD_STAKEHOLDER_FROM_THE_BRIEF``). Precedent stays for a brief that
   leaves it out.

.. dec:: A closing word is a defect only where it closes a set
   :id: DEC_CLOSING_WORD_NAMES_ITS_SET
   :dec_status: accepted
   :decided_on: 2026-09-27
   :supported_by: EVD_CLOSING_WORDS_BANNED_OUTRIGHT, EVD_CLOSING_WORD_NAMES_ITS_SET
   :statement: A development workflow's judge shall count a closing word as a defect only when it names the set of options or mechanisms the sentence declares complete.

   A script finds the words, and most uses close nothing: "each time", "how
   many calls each operation takes". Banned outright, they stopped goals at
   the pass limit (``EVD_CLOSING_WORDS_BANNED_OUTRIGHT``). The judge's form
   now asks which set the sentence declares complete, and a finding whose
   answer is NONE is not counted (``EVD_CLOSING_WORD_NAMES_ITS_SET``).
