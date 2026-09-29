# Custom transform implementation

## Scope

Sketchpad defines a custom transform by selecting a pre-image point followed by
an image point that depends on it. Applying the named transform to another point
substitutes that point for the pre-image and reevaluates the same construction.
The reference center also specifies that custom transforms can be renamed,
reordered, and removed:

- [Define Custom Transform](https://keyonline.keypress.com/gsp-reference-center/menus_trans_custdefine.htm)
- [Edit Custom Transforms](https://keyonline.keypress.com/gsp-reference-center/menus_trans_custedit.htm)

The port implements that bounded point case. It does not claim shape outputs,
multiple inputs, or native persistence of the definition list.

## Representation

`Document::custom_transform_carried(seed, output)` derives the affected slice
from the authoritative construction graph. A forward pass identifies objects
whose values depend on the seed; a backward walk from the chosen output removes
unrelated branches. Native construction order supplies the topological order.
Independent operands remain external inputs and are frozen by the existing
mapped evaluator.

Applying a definition calls `Document::add_mapped_element`. This is the existing
native code-101 representation, so custom transforms do not add a second
geometry evaluator or a new file-format family. The definition is revalidated
on every application, so edits to the defining construction are live and broken
dependencies disable the menu command.

The desktop definition list is currently document-session state. New/Open clears
it. A save that compacts or reorders object IDs also clears it explicitly instead
of risking stale references. Already applied mapped points are ordinary document
objects and persist normally.

## Verification

One structural regression constructs a midpoint relationship, derives the exact
`[segment, midpoint]` carried graph, maps a replacement point, checks the expected
position, encodes, reparses, and confirms the native mapped-element kind and
position.

A headless replay generated a seven-object sketch containing two defining
points, their segment and midpoint, a replacement point, the code-101 mapped
point, and a child segment attached to the mapped result. The port reopened all
seven objects. Bundled Sketchpad 5 then opened and saved the 2,236-byte file
without changing a byte (SHA-256 before and after:
`a55913e3b48950e72b45d1a7006f176121a18683624b8b6f49f2ddc662466399`).

Workspace formatting, Clippy with warnings denied, and the full test suite are
the release gates for this change.
