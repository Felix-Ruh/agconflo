====================
Code in agconflo-lua
====================

Where each component requirement allocated to ``agconflo-lua`` is met in its
source. Nothing below is written here: every implementation comes from a
one-line marker in ``crates/agconflo-lua/src``, placed on the item that does
the work::

  // @<title>,<IMPL id>,impl,[<component requirement ids>],[<decision or note ids>]
  // @<title>,<TRACE id>,trace,[],[<decision or note ids>]

The identifier is ``IMPL_`` followed by the module and a name for what the code
does there, the way a test case's identifier follows its test's path, so the
module says which component the code belongs to. One requirement may be met in
several places, and each place carries its own marker: that is why the markers
are one line each rather than references to a need written here, which keep
only the first place (``DEC_IMPL_FROM_MARKERS``).

The second list, which may be left out, names the decisions and notes the code
follows (``DEC_CODE_FOLLOWS_BY_MARKER``). A ``trace`` marker is for code that
follows one and meets no component requirement (``DEC_TRACE_MARKERS``). A note
is written in this document, as a ``code_note``, when an explanation belongs to
one piece of this crate's code alone (``DEC_NOTES_BESIDE_THE_CRATE``).

Each implementation's and trace's ``code_url`` is a link into the repository
at the commit being checked, recorded when the project is indexed rather than
written by anyone. A marker naming a requirement that does not exist is a dead link, and
fails the check like any other. A requirement that no marker names is not an
error, since code comes after its requirement; the review report
``scripts/reports/unimplemented.cypher`` lists them.

.. code_note:: The script host's default limits
   :id: NOTE_HOST_DEFAULT_LIMITS

   Ten million instructions, 64 MiB and one model call: a few tens of
   milliseconds of work, several thousand times what an empty state holds, and
   a run whose model calls are bounded by its step budget. None of the numbers
   is a requirement; each is room for a script that assembles text and asks one
   question. The model call default is ``DEC_EVERY_TURN_COUNTED``'s.

.. code_note:: How the script host tells a failure's kind
   :id: NOTE_HOST_FAILURE_ORDER

   Every limit, and a failed model call, reaches a script as an ordinary Lua
   error, and its message is not what says which it was. So what a call to the
   host could not complete is kept outside the script, and flags are asked
   before the error is: the instruction limit first, since a script over it may
   have been anywhere, a model call's aftermath included; then the model call
   limit; then a failure a host function recorded. Only then is the error read,
   and a memory error is the memory limit whatever raised it. Anything else is
   the script's own error, kept whole, since its document, line and traceback
   are what its author reads. A limit is never reported as a script error, nor
   the other way round.

.. code_note:: Why the host's functions own what they use
   :id: NOTE_HOST_OWNED_HANDLES

   A model call is awaited, and a function scoped to a borrow cannot be. So each
   host function owns what it needs - the identifier source, the roster, the
   call counts - rather than borrowing it for the activation. The run cannot be
   owned that way, since it borrows its workflow: the host leaves what it asks
   of the run in a mailbox and waits, and whatever polls the script answers
   from the run it holds (``DEC_RUN_IS_DRIVEN``). The yield is to the driver, as
   the call is a step of the run.

   A scripted run lends its caller's identifier source to the scripts the same
   way, and puts it back advanced past every identifier the run issued however
   the run ends, an early return included.

.. src-trace::
   :project: agconflo-lua
