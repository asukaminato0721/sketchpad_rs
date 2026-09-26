# Unified transformation model

The internal model will represent a transformed object by its source object,
element type, and transformation definition. The current 60 variants multiply
six element families by native transformation modes. Shared operations replace
that product: translation, rotation, dilation, and reflection.

Transformation definitions retain live references to centers, mirrors, vectors,
measured angles, ratios, and calculation objects. Matrices are evaluated from
those definitions. Each transformation remains a separate object in the source
graph, so intermediate objects keep their IDs, styles, visibility, dependencies,
and undefined states. A matrix sequence is an evaluation view of that graph.

Element types distinguish points, segments, rays, infinite lines, circles, and
polygon interiors. This small discriminator lets current classifiers retain
constant-time behavior without introducing operation-specific element variants.
Native subtype validation must establish that the source and target agree.

Native wire encodings remain separate from semantic operations. They preserve
record codes, reference ordering, literal unit scales and magnitudes, legacy
headers, caches, styles, and polygon counts/ranks. The adapter must not silently
widen support to unverified source families or layouts.

Migration sequence:

1. Project all existing variants into shared definitions and wire descriptors.
2. Share geometry evaluation and dependency/reference traversal while retaining
   existing importer, writer, and regression goldens.
3. Move each operation family to the generic object representation and migrate
   import/export, validation, editor commands, plotting, rendering, and picking.
4. Remove compatibility variants after all callers and native fixtures agree.

Checks include native byte goldens, live parameter changes and domain recovery,
zero-scale ancestors, ray extent and line extrapolation, inherited styles,
clipboard remapping, dependent deletion, and iteration substitution. Combining
matrices must retain intermediate suppression; point and numeric substitution
must continue to follow each existing iteration family's verified semantics.

At the start of migration the current full gate passes 782 tests. All 1599 corpus
files round-trip byte-identically; 108817 object slots remain unsupported,
including 5356 embedded tool prototypes. These are baseline measurements, not a
completion claim.

Stage 2 native authoring now uses `document::transform_wire::encode_group` for
all 60 compatibility variants. The dispatcher reads the shared element, native
flavor, ordered references, and literal roles. It preserves existing native
templates, exact cardinal caches, literal magnitudes/scales, signed polygon
drawing orders, subtype counts, point labels, visibility, and vector polygon
locus metadata. Imported raw records remain owned by the preserving serializer;
the adapter does not introduce additional supported families.

The repeated transform branches in `encode.rs` have been removed. Existing
transform, translation, and polygon regression suites pass (23, 15, and 17 tests),
including authored native goldens, live numeric edits, clipboard remapping,
inherited styles, and signed drawing orders. Focused wire tests also cover
nondefault literal scales, signed zero, duplicate reference order, and native
cardinal caches. The legacy variants remain a temporary compatibility adapter;
generic object storage and decoder migration are subsequent steps.

Stage 2 save validation now projects each live transform through the same
definition and delegates to `document::transform_validation::validate_transform`.
The capability matrix is keyed by element and native flavor, rather than 60
variant-specific branches. It retains primitive-only sources for unproved
operation/source combinations, previously proved segment/circle/polygon chains,
the ten-kind ray/line mode boundary, literal unit scales, finite fields, and
numeric source-unit categories. Generic ID, dependency, style, and page policies
remain in the document save validator; multi-page serialization remains separate.

Imported nonnumeric transforms retain raw-save policy without requiring
evaluated geometry. Imported numeric rotation/dilation retains its existing
typed direct-parent checks. New marked translations intentionally permit
undefined live distance/source geometry after category checks. New numeric
dilation retains its existing distinction: point output must be defined,
while finite-ratio segment/circle/interior targets can remain collapsed or
undefined. Ray/line targets preserve native zero-factor, undefined live numeric
value, and suppressed-ancestor states, while still requiring a defined center.
Successful shared evaluation is required only where the prior validator
required available target geometry. This refactor adds no independent
cross-page capability or schema changes.

The consolidated validator removes about 500 repeated lines. Four dedicated
policy regressions cover all 60 compatibility variants, both mixed translation
flavors, all 14 wire flavors and six elements, imported/new capability
differences, invalid references/modes/units/literals, undefined marked values,
and zero dilation. The existing 51 transform-filtered library regressions pass,
including native byte goldens, rejected unproved chains, live domain recovery,
copy/deletion, inherited styles, and undefined straight ancestors. A full
combined gate remains required after the coordinated Stage 2 integration.

The shared model now supplies point/straight/circle/polygon classification,
source vertex counts, default styles, and inherited polygon/ray/line styles.
Locus and plotted-path capability filters still restrict the same native flavors.
Geometry, writer, and validation integration removes repeated per-element
implementations while the legacy enum adapter remains temporary. The generic
Kind representation and final removal of those variants are still pending.

Dependency traversal now reads the shared ordered native references for all 60
compatibility variants, including repeated references and malformed zero IDs.
Direct iteration evaluation also consumes the shared operation and expression
model. Its substitution resolver retains Native behavior: direct seed points
move, while measured numeric values and reflection mirror geometry remain
pinned. Simultaneous mapped iterations retain their separate sample-document
resolver so dependent calculations re-evaluate in the substituted graph. All
60 existing library iteration regressions and 21 editor iteration regressions
pass after this consumer migration. The shared `mapped_segment_binding`
predicate retains only primitive segments and vector-translated segment
bindings, allowing the decoder, writer, and validator to consume the same
existing policy without widening iteration capabilities.

Circle/polygon rendering, polygon picking, moved-polygon markers, polygon locus
header metadata, and iteration-binding wire classification now read exact
element/flavor predicates. Primitive source restrictions are retained. The
rendering/picking/clipboard focused gates pass 68 tests; the combined gate is
still needed after geometry corrections. Decoder construction, command
constructors, Kind storage and remapping remain the next representation cutover.
See `transform_generic_migration_plan.md` for the native import acceptance
boundaries that must be retained during that cutover.

The combined all-targets gate after this integration passes 805 tests:
642 library, 112 editor, 42 numeric Native import, four coverage, two plotting,
and three value/equation tests. The gate exits successfully. Corpus checkpoint
14 independently retains all 1599 byte-identical round trips and 108783
unsupported object slots in the same drawing/tool scope. The Native-proved
line-circle branch/extent and derived-line parameter corrections pass together
with iteration depth and horizontal-slope caption precision regressions.
Generic storage and removal of the 60 compatibility variants remain pending;
these passing gates establish the consumer migration baseline.
