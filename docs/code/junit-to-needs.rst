======================
Code in junit-to-needs
======================

The code of the test result importer and the decisions it follows. The
importer is development tooling and meets no component requirement, so each
of its markers is a trace (``DEC_TRACE_MARKERS``), placed on the item that
follows the decision::

  // @<title>,<TRACE id>,trace,[],[<decision or note ids>]

The identifier is ``TRACE_`` followed by the module and a name for what the code
does there. Each trace's ``code_url`` is a link into the repository at the
commit being checked, recorded when the project is indexed. A marker naming a
decision that does not exist is a dead link, and fails the check like any
other.

.. code_note:: How the importer's generated tests are built
   :id: NOTE_IMPORT_GENERATED_TESTS

   The property tests in ``import.rs`` check the importer against reports
   generated from parts. The id each generated test should produce is put
   together from those parts, never by splitting the names the importer reads,
   so that the expectation cannot agree with the importer by construction.

   Every generated set of tests starts with one test in each traced crate, so no
   generated report is refused for a crate with no tests, which would test the
   refusal rather than the import. Two traced paths joining to one id are left
   out of the generated sets, since that is a refusal of its own, and
   ``colliding_ids`` tests it by hand. The untraced crate's tests may repeat
   freely, since none of them is read.

.. src-trace::
   :project: junit-to-needs
