====================================
Evidence about development workflows
====================================

Measurements behind ``decisions/workflows``, taken while building the first
development workflow Agconflo runs on itself (``STKH_SELF_HOSTING``): one that
records stakeholder goals the maintainer has approved. It lives in
``workflows/record-goals``, where its README says how to run it.

Every run below reached a provider: OpenRouter, through a key the maintainer
allowed for testing, from a cloud development container. Each run was given a
brief asking for a past pull request's goals from that pull request's parent
commit, and its result was compared with what the pull request merged: #55,
two goals, from ``1f67ee4``, and #29, three goals in two files, from
``437f302``. The runs are numbered by the workflow's version. Versions 1 and 2
drafted the change in one or a few model calls, and their output is on the
branches ``bootstrap/routing-goal-v1`` and ``-v2``. Versions 3 to 7 were fitted
to #55, and versions 8 on serve any such brief. Each finding is of the day it
was taken, with the model named in it.

.. evd:: Offered no tools, a weak model wrote its tool calls out as its answer
   :id: EVD_WEAK_MODEL_WRITES_CALL_MARKUP
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: In version 3, deepseek-v4.1-flash answered 2 of 9 model calls with its own tool-call markup as text, both from steps offered no tools, and a step offered read_file and write_file called bash, which failed the run after 43 activations.

   The markup was the model's own format, ``<｜｜DSML｜｜ invoke
   name="bash">``, sent as the answer's text: the step that was to copy the
   brief's statements out answered with a call to ``shell``, and the step
   that was to answer seven decisions with a call to ``bash``. Both answers
   became outputs, and the steps after them worked from them. The step that
   inserted the drafts read five files, two at paths that do not exist, and
   then called ``bash``. A call to a tool not offered fails its activation
   (``DEC_MALFORMED_CALL_FAILS``), and the run ended there.

.. evd:: Without tools to call, the markup and the calls stopped
   :id: EVD_NO_TOOLS_NO_MARKUP
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: From version 4 to version 9, with at most one review step offered a tool and every prompt naming its tools or saying it had none, 406 model answers over 1489 activations held no tool-call markup and no run failed on a call.

   Counted over every answer each run's record holds. Version 4 moved the
   file work out of the model: a command step reads the file, the model
   names where each goal goes, a script splices, and a wired
   ``write_file`` step writes. From version 8 no model step is offered a
   tool at all. The request that offers tools was captured against a stub
   too: each tool went out as an OpenAI function with its description and a
   string schema per parameter, so the markup was not a malformed offer.

.. evd:: A judge outside the loop that owns what it faults cannot get it fixed
   :id: EVD_JUDGED_STEP_OUTSIDE_THE_LOOP
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: In version 6 the judge found defects in the file's introduction on all 3 passes while the step writing the introduction ran once before the loop, so each pass sent back only the drafts and reported the same defects as not fixed.

   The introduction step sat before the judge's loop with a validator of its
   own, and the judge routed back to the drafts alone. Version 7 moved it
   inside the loop, taking the judge's verdict as its feedback, and a stub run
   showed it running again beside the drafts on the second pass.

.. evd:: A judge weighs only what the review raised
   :id: EVD_JUDGE_WEIGHS_WHAT_WAS_RAISED
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: Version 7 was accepted on its first pass, its judge finding none of 7 findings a defect, while its change denied a relation #55 draws and cited none of the three decisions its brief named, which no step had raised.

   Its judge was then ``deepseek/deepseek-v4-pro-0813``, a stronger model than
   the steps it judged, which was later withdrawn. The change read "Neither
   needs the other: a workflow can route without repeating any part", where
   #55 says the repetition goal leans on routing, and "can hold alongside
   ``STKH_ONE_OUTPUT`` without either needing the other", where #55 says
   one output is what makes the deciding node's output its decision. The
   review then was one step writing 8 to 12 hypotheses and one verifying them.

.. evd:: Without a rule for claims and relations, a weak judge upheld nothing
   :id: EVD_JUDGE_WITHOUT_A_RULE_TO_QUOTE
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: In version 8 the flash judge, asked to quote the rule each finding breaks or answer NONE, upheld 0 of 43 findings over 5 goals, and for 18 of the 36 unsupported-claim and wrong-relation findings it found no rule to quote.

   The form said a finding whose rule is NONE is not a defect, and no
   sentence of the requirement rules in ``AGENTS.md`` or the README forbids
   a claim no need supports or a need described wrongly. For the other 18 of
   those findings it quoted a rule and answered no. The 7 other findings were
   closing words, where it quoted the right rule and still passed "every
   possible route must be settled when the workflow is written".

.. evd:: With a policy sentence per kind of finding, the weak judge sent goals back
   :id: EVD_JUDGE_WITH_A_POLICY
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: In version 9, given one policy sentence to quote per kind of finding, the flash judge upheld 21 of 44 findings, among them a need described wrongly and a missing point of the maintainer's note, and sent 4 of 5 goals back at least once.

   The wrong relation it upheld: the draft made ``STKH_MACHINE_AUTHORING``
   the one surface for an interface, where that need says an agent creates
   and modifies workflows through tool calls. The missing point was "The
   interface widens STKH_MACHINE_AUTHORING", found by a step that checks each
   sentence of the note against the draft; the next pass put it back.

.. evd:: A policy banning closing words outright stopped goals at the pass limit
   :id: EVD_CLOSING_WORDS_BANNED_OUTRIGHT
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: In version 9 a policy forbidding only, each, every, exactly and no other made the flash judge uphold phrases such as each time; all 3 goals stopped at the pass limit had such findings open, one nothing else.

   ``AGENTS.md`` allows those words where the parent closes the world too;
   what it forbids is a claim that declares a set of mechanisms complete.
   The judge upheld "a private shape each time" and "how many calls each
   operation takes", neither of which declares any set complete. The run on
   #29 took 53 minutes and 256 activations; the two goals stopped there also
   kept wrong relations open.

.. evd:: The record-goals workflow chose the merged file for every goal of two pull requests
   :id: EVD_RECORD_GOALS_ON_TWO_PULL_REQUESTS
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: Run unchanged on #55 and #29 from their parent commits, versions 8 and 9 put each of the 5 goals in the file its merged pull request chose, and each change passed ubc check and ubc format at its parent.

   #29 split its goals over ``authoring.rst`` and ``execution.rst``, and the
   workflow chose the same split, choosing a file per goal inside its loop.
   Both versions reached the approval step on both pull requests, and their
   final step found every statement recorded as given, nothing changed
   outside ``docs/stakeholder`` and every need the brief named cited.

.. evd:: Chosen from the definition alone, the stakeholder matched once in five
   :id: EVD_STAKEHOLDER_FROM_THE_DEFINITION
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: Choosing each goal's stakeholder from the definition in ubproject.toml, the flash model matched the merged pull requests for 1 of 5 goals in version 8, and following an existing goal as precedent, for 3 of 5 in version 9.

   In version 8 it chose ``agent`` for every goal, reasoning that the node
   involved is a model. The definition reads "someone running workflows, an
   LLM operating or authoring them, or someone working on Agconflo itself",
   and a goal about what a model may do reads as the second. The
   maintainer chose ``user`` for four of the five; #29's commit says its goals
   were "worded with the stakeholder".

.. evd:: Given in the brief, every stakeholder was recorded as the maintainer chose
   :id: EVD_STAKEHOLDER_FROM_THE_BRIEF
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: In version 10, with each goal's stakeholder given in the brief, all 5 goals of #55 and #29 were recorded with the stakeholder their merged pull request has, and the final step checked each.

   The brief carries it beside the statement, ``(stakeholder: user)``, as the
   maintainer approves both together. The naming step still names a precedent,
   and its check holds the stakeholder to the one given.

.. evd:: Asked which set a closing word closes, the judge let most of them stand
   :id: EVD_CLOSING_WORD_NAMES_ITS_SET
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: In version 10 the judge's form asked which set each closing word declares complete; for 15 of 19 closing-word findings it named none and they were not counted, and both goals of #55 were accepted on their first pass in 9 minutes.

   Version 9 had stopped #55's second goal at the pass limit on the word
   *each* alone (``EVD_CLOSING_WORDS_BANNED_OUTRIGHT``). The policy sentence
   is now the rule as ``AGENTS.md`` gives it: closing words are allowed
   where the parent closes the world too.

.. evd:: A need the brief names can go uncited by every goal
   :id: EVD_BRIEF_NEED_LEFT_UNCITED
   :evd_kind: measurement
   :observed_on: 2026-09-27
   :observation: Version 10 on #29 ended not ready: no body cited EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS, which the brief names, and two goals stopped at the pass limit with a sentence of the maintainer's note contradicted or missing.

   The yield goal's neighbours step chose four needs and not that one,
   although it was told to take first any need the brief names that concerns
   the goal. The note step then found the note's reason for the yield
   missing on each pass, and the drafts did not put it back within three. The
   final step caught the uncited need, as it is there to. Choosing which of
   the brief's needs concern a goal is one more question for a step of its
   own.

.. evd:: Rewritten to share a module, the scripts ran as the generated ones did
   :id: EVD_SCRIPTS_REWRITTEN_ALIKE
   :evd_kind: measurement
   :observed_on: 2026-10-07
   :observation: From 1f67ee4 on #55's brief, against a stub model answering from the prompt alone, the hand-written scripts sent the generated ones' 115 prompts byte for byte and left the same record, 2,158,208 bytes over 199 activations.

   Both runs were on the engine at ``cf97326``, with the same images and a
   fresh clone each. The stub chose a stakeholder file and the first goal in
   it to follow, wrote a directive ``ubc`` refuses, and answered everything
   else ``NONE``, so most checks failed and asked again, and every pass
   failed ``ubc``. The judge sent each goal back twice, went on to the second
   goal, and then on to the final steps; both runs stopped at ``approval``,
   exit 3, in 42 and 41 seconds, with the same change staged.

   It does not reach the judge's model, which is asked only of findings once
   ``ubc`` passes, and it says nothing of a real model's answers, which a
   stub cannot stand in for.

.. evd:: With a router passing the judge's state on, the run went as before
   :id: EVD_JUDGE_STATE_PASSED_ON_ALIKE
   :evd_kind: measurement
   :observed_on: 2026-10-07
   :observation: With the judge a step and a router after it passing its output on, the run of EVD_SCRIPTS_REWRITTEN_ALIKE on the engine at a3f676d sent the same 115 prompts, took the same routes and stopped at approval with the same step and change.

   The same clone, brief, stub and images as ``EVD_SCRIPTS_REWRITTEN_ALIKE``.
   It spent 205 activations where that spent 199: the router's six. What
   ``approval`` was given, the judge's last verdict among it, printed byte
   for byte as before, and ``agconflo check`` refused the judge as a router
   with an output on this engine, at its declared output.

   On #29's brief, three goals, from ``437f302``, the generated scripts on
   ``cf97326`` and these on ``a3f676d`` sent the same 169 prompts byte for
   byte, went back eight times and on once, and stopped at ``approval`` with
   the same step and the same change; 286 activations, and 295 with the
   router's nine.

.. evd:: Rewritten, the workflow recorded both pull requests' goals on the weak model
   :id: EVD_REWRITE_ON_THE_WEAK_MODEL
   :evd_kind: measurement
   :observed_on: 2026-10-08
   :observation: On deepseek-v4.1-flash, the hand-written workflow recorded all 5 goals of #55 and #29 with the statement, stakeholder and file their merged pull requests have, its final check passed both, and each kept one goal at the pass limit.

   Run with ``prepare.sh`` and ``run.sh`` as the README says, on the engine
   and scripts at ``de25d76``, each brief from its parent commit, every role
   played by ``deepseek/deepseek-v4.1-flash`` through OpenRouter with a key
   the maintainer allowed for these runs. Both stopped at ``approval``,
   exit 3.

   #55, from ``1f67ee4``: 28 minutes and 175 activations. Goal 1 was sent
   back twice and went on after pass 3 with one closing word upheld, "every
   branch that follows"; goal 2 was accepted on pass 2. Version 10 had
   accepted both on their first pass in 9 minutes.

   #29, from ``437f302``: 47 minutes and 235 activations. Goals 1 and 2 were
   accepted on pass 2, goal 3 went on after pass 3 with 1 of 9 findings
   upheld, and the final check found every need the brief names cited,
   where version 10 left one uncited (``EVD_BRIEF_NEED_LEFT_UNCITED``).

   The prompts are the generated scripts' byte for byte
   (``EVD_SCRIPTS_REWRITTEN_ALIKE``), so where these runs differ from
   version 10's they differ as one run of the model differs from the next.
   One run of each brief says nothing of which outcome is the usual one.
