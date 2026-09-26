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
