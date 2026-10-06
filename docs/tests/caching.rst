==============================
Caching a next turn test cases
==============================

How each requirement in ``components/caching`` and ``features/caching`` is
to be checked. Results are never written here: they are imported from the
test runner.

A case's id is the path of the Rust test that implements it, uppercased. The
roster's cases live in ``agconflo-lua``'s module ``models``, the feature's in
its module ``scripted``.

No case reaches a provider. Each sends to the stub on the loopback interface
that answers in OpenAI's or Anthropic's format, as ``tests/models`` does, and
reads the marks from what the stub was sent. What a provider then reads from
its cache is beyond a stub; what a request needs for it to - a beginning
alike, and a mark where a provider wants one - is what these show.
``TEST_MODELS_CONTINUATION_SENDS_CONTEXTS_WHOLE`` sends its continued windows
in Anthropic's format offering a node type, so it holds the texts unchanged
under marks too.

.. test_case:: A window offering node types is marked at its end and before its last answer
   :id: TEST_MODELS_CONTINUATION_MARKED_FOR_CACHE
   :verifies: CREQ_ROSTER_MARKS_CONTINUATION
   :test_kind: property
   :coverage: full

   For any number of turns from none to three, each answer with text or
   none and making from one to twelve calls, sent in Anthropic's format
   offering a node type: the messages carrying a mark are exactly the last
   and, after a first turn, the one before the last answer, each mark on its
   message's last block. The same windows sent in OpenAI's format to a model
   that caches by itself carry no field about caching - the control.

   Catches: only the last message marked, the answer marked, a third message
   marked, a mark not reaching Anthropic's format.

.. test_case:: A window offering nothing carries no field about caching
   :id: TEST_MODELS_UNOFFERED_WINDOW_UNMARKED
   :verifies: CREQ_ROSTER_UNOFFERED_UNMARKED
   :test_kind: positive
   :coverage: full

   A first window and a continued one, each offering nothing, in both
   formats: no request carries ``cache_control``, ``prompt_cache`` or any
   other field about caching. The continued window offering a node type
   carries two marks in Anthropic's format - the control.

   Catches: a call offering nothing marked, the option set on the whole
   request.

.. test_case:: OpenAI's marking models keep the mark on the prompt and lose it on a result
   :id: TEST_MODELS_EXPLICIT_OPENAI_MARKS_THE_PROMPT_ONLY
   :verifies: CREQ_ROSTER_MARKS_CONTINUATION
   :test_kind: error_path
   :coverage: partial

   In OpenAI's format to ``gpt-5.6``, offering a node type: a first window
   carries ``prompt_cache_breakpoint`` on its prompt; a second carries it on
   the prompt and on nothing else, its mark on the result dropped; a third
   carries none, both its marks having fallen on results
   (``EVD_CACHE_MARK_ON_A_TOOL_RESULT``).

   Pins the shortfall ``DEC_CACHE_MARKED_BEFORE_A_CONTINUATION`` records, so
   that a ``genai`` keeping or losing more of the marks fails here rather
   than changing what a provider is sent without a word.

.. test_case:: Each turn begins with the turn before, marked where it ended
   :id: TEST_SCRIPTED_CONTINUATION_BEGINS_WITH_THE_TURN_BEFORE
   :verifies: FEAT_CONTINUATION_CACHED
   :test_kind: positive
   :coverage: partial

   A script whose model calls ``lookup`` on two turns and then answers, run
   in Anthropic's format: each request after the first begins with every
   message of the one before, text for text, and carries marks on the
   message the one before ended with and on its own last.
