# Split/Merge analysis

Sketchpad 5 documents Split as detaching a midpoint, point on a path,
intersection, or plotted point from its parents so that it becomes independent.
The port's first bounded implementation rewrites such a point's object group in
place. Keeping the native object ordinal means existing children require no
reference remapping.

The focused engine regression creates two endpoints, their segment, its
midpoint, a second free point, and a child segment from the midpoint. It saves
and reopens the construction, splits object 4, moves a former parent, and checks
that object 4 stays fixed while object 6 still depends on it. The second save
reopens with object 4 decoded as `Kind::Free`.

The same six-object output was opened and saved with the bundled Sketchpad 5
under Wine. The native save remained byte-identical at 2,168 bytes, providing a
cross-open check for the replacement free-point group and its surviving child
reference.

Merge Points supports the complementary bounded case: the first-selected point
must be independent and the second selection is the target. The transaction
rewrites every supported child reference to the target, rejects dependency
cycles or unsavable layouts, removes the source, and validates a save/reopen
before committing. A focused regression covers a source point whose child
segment remains attached to the surviving target after object-ID compaction.
The generated three-object result was also opened and saved using bundled
Sketchpad 5 under Wine. Its native save remained byte-identical at 1,968 bytes.

Merging a point to a path, merging text or pictures, transformed-point
detachment, independent-point child duplication, and multi-page structural
edits remain unimplemented.
