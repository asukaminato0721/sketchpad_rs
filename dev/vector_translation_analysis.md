# Mark Vector and vector translation

`tests/transform_vector_replay.txt` builds a sketch with vector points at
`(150, 120)` and `(220, 130)`, then translates a source point at `(100, 200)`.
The resulting point is `(170, 210)`. Moving the vector endpoint to `(250, 130)`
moves the result to `(200, 210)` without rebuilding the translation.

`dev/vector_wine_probe.sh /tmp/sketchpad_vector_base.gsp` selects the vector's
two points in Sketchpad 5, runs Transform > Mark Vector, selects the source,
and uses the marked option in Transform > Translate. Its saved result is
`tests/fixtures/vector_translation_original.gsp`. Saving immediately after
Mark Vector produces the same bytes as the unmarked source file; the mark is
session state.

The native translated point has object header code 16, a point style, and a
`0x07d2` reference list `[source, vector start, vector end]`. There is no
additional constraint payload. The port reads and writes this object, keeps
the vector dependencies, and evaluates `source + end - start`. Its editor
offers Mark Vector and a marked-vector choice in Translate. The marked pair
is shown in the sidebar.

The generated files are
`tests/fixtures/vector_translation_generated.gsp` and
`tests/fixtures/vector_translation_moved_generated.gsp`. Sketchpad 5 opened
and re-saved both byte-identically. This covers point sources and a vector
defined by two points. Translation of other object types remains outside this
implementation.
