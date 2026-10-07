# record-goals

A development workflow Agconflo runs on itself (`STKH_SELF_HOSTING`): given a
brief listing stakeholder goals the maintainer has approved, it records each as
a `stkh_req` directive in the right file under `docs/stakeholder/`, checks the
change, and stops at an approval step with the change staged. It opens nothing,
pushes nothing and merges nothing.

Every model step runs on `deepseek-v4.1-flash`. The workflow is the structure
that makes a weak model do this; the reasons for that structure are decisions
in the requirements graph, `docs/decisions/workflows.rst`, and the measurements
behind them are in `docs/evidence/workflows.rst`. Ask the graph:

    tools/ubc query cypher --project docs --strict 'MATCH (d:dec)-[:supported_by]->(e:evd) WHERE d.docname CONTAINS "workflows" RETURN d.id, e.id'

## The brief

The only input. Each approved goal is a numbered line quoting its statement,
optionally with its stakeholder; an optional maintainer's note follows:

    1. "Agconflo shall let a node decide which of the branches after it a run takes." (stakeholder: user)
    2. "Agconflo shall let a workflow repeat part of itself until a node decides it is done." (stakeholder: user)

    Maintainer's note: existing decisions already discuss loops and routers: DEC_BACK_EDGES_ALLOWED, ...

A statement is any quoted text beginning `Agconflo shall`. A goal without a
stakeholder gets one by precedent: the stakeholder of the existing goal most
like it. The note is checked sentence by sentence against each draft, and
every need id it names must be cited by some body. `briefs/` holds the briefs
for #55 and #29, the two past pull requests the workflow is measured on.

## Running it

From the repository root, with Docker running and `cargo build` done, on
Linux or on Windows under Git Bash:

    sh workflows/record-goals/prepare.sh ../run-55 1f67ee4 [certificates]
    sh workflows/record-goals/run.sh ../run-55 workflows/record-goals/briefs/pr55.txt

`run.sh` needs `OPEN_ROUTER_API_KEY` in its environment; give it to that one
command rather than exporting it to the shell.

`prepare.sh` builds the two images the tool steps run in - the ubc one with
ubc's Linux build, downloaded at the version and checksum
`scripts/get-ubc.sh` pins - clones the repository from GitHub at the given
commit into `<run>/work` - ubc grants its licence by the remote, so a clone of
a local checkout will not do - and fills in what this machine decides: the
image ids and the work folder. Behind a proxy that intercepts TLS, pass its
certificates as the third argument.

`run.sh` keeps the record in `<run>/run.toml` and stops where `agconflo`
stops; exit 3 is the approval step. Then:

    git -C ../run-55/work diff --cached                    # the change

The record is the run's trace: each step's passes in order, what each model
was shown and answered, each output with the inputs it was made from, and
where the loop went after each verdict. #55 was measured from `1f67ee4` and
#29 from `437f302`; compare each with what it merged, `git show a3a4066` and
`git show 3778551`.

## What it does

**Once.** The statements and stakeholders from the brief; the stakeholder
files with their goals; every goal and every need the brief names, from the
graph; the sections of `AGENTS.md` and the README whose headings mention
requirements; the repository's definition of the stakeholders; an example
goal.

**Per goal, in one loop.** The judge's output is the loop's state: which goal,
which pass, and the feedback. The router after it, `next`, passes that state
back round, to `head` and to the judge, while it names a goal and a pass, and
on to the final check once it names none; a router makes nothing of its own.
What comes before the judge's first output is declared in `flow.toml` on the
two edges back: to `head`, goal 1 on pass 1, and to the judge, a start with no
notes. Each attempt starts from git's index,
where the accepted goals are staged. One step per question, each checking its
own answer and asking again with the reasons, at most three times:

| Step | Asks | Checked by script |
| --- | --- | --- |
| `pick_file` | which file the goal belongs in | a listed file |
| `keywords` | words to search the decisions for | 4 to 8 lowercase words |
| `neighbours` | 2 to 4 needs the body should name, each with a quoted sentence | ids exist; the sentence is found where it was quoted from |
| `naming` | id, title, precedent, stakeholder | id new; stakeholder as given, or the precedent's |
| `place` | after which goal of the file | a goal of the file |
| `intro` | the introduction with the goal's theme, or UNCHANGED | same opening; lowercase clauses; width |
| `draft` | the directive | shape, id, title, stakeholder, statement exact; names every neighbour and no other; double backquotes |

A script splices the draft and the introduction into the file, a wired
`write_file` step writes it, and ubc checks the project. Then the review, one
kind of finding per step: closing words (a script), claims no need supports,
each sentence naming a need paired with that need's statement from the graph,
and each sentence of the maintainer's note. The judge fills in a form per
finding and counts it a defect only when it quotes the rule it breaks, word
for word, from the task, the rules or its review policy. A defect sends the
goal back, at most three passes; otherwise the next goal starts.

**Once, at the end.** ubc over the whole change, every statement recorded with
its given stakeholder, nothing changed outside `docs/stakeholder/`, every need
the brief names cited. A model writes the pull request's description, and the
run waits at `approval` for a person.

## Changing it

The documents are written by hand (`DEC_WORKFLOW_SCRIPTS_SHARE_A_MODULE`).
What the scripts share - the checks, the step that asks a model and asks
again with the reasons, and the text every prompt shares - is
`lib/help.lua`, which a script reaches with `require('help')`. A model step
is `m_<name>.lua`, a script step `s_<name>.lua`, and a script writing a
command for a tool step `s_cmd_<name>.lua`. Two values stay placeholders in
the committed files, `@UBC_IMAGE@` and the ones in `grants.template.toml`,
which `prepare.sh` fills. Run a change on both briefs before relying on it
(`DEC_WORKFLOW_PER_KIND_OF_CHANGE`), and against a stub model first: a local
server answering `/v1/chat/completions` in the OpenAI shape, named in the run's
`models.toml` in place of the provider, costs nothing and finds wiring faults.

## What it does not do yet

- A need the brief names can go uncited: on #29, version 10 ended not ready
  because no body cited the measurement its note names
  (`EVD_BRIEF_NEED_LEFT_UNCITED`), where the rewrite's run cited it
  (`EVD_REWRITE_ON_THE_WEAK_MODEL`). Which of the brief's needs concern a
  goal is still left to the step choosing its neighbours.
- A goal can go on at the pass limit with a defect open: each run of the
  rewrite on the weak model kept one (`EVD_REWRITE_ON_THE_WEAK_MODEL`). The
  final step then says the change is not ready, and a person decides.
- The bodies relate a goal to its neighbours need by need; the merged pull
  requests argue one or two relations at length.
- Placement within a file is its own guess; #55 put its goals a few goals
  earlier than the workflow does.
- A run on three goals takes up to an hour and a few hundred activations.
