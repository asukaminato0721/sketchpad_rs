# Custom transform implementation

## Scope

Sketchpad defines a custom transform by selecting a pre-image point followed by
an image point that depends on it. Applying the named transform to another point
substitutes that point for the pre-image and reevaluates the same construction.
The reference center also specifies that custom transforms can be renamed,
reordered, and removed:

- [Define Custom Transform](https://keyonline.keypress.com/gsp-reference-center/menus_trans_custdefine.htm)
- [Edit Custom Transforms](https://keyonline.keypress.com/gsp-reference-center/menus_trans_custedit.htm)

The port implements that bounded point case and imports Native's single-page
definition metadata. It does not claim shape outputs, multiple inputs, or
writing new or edited definitions.

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

Native stores each named definition in a document-level `0x2331` record: seed
ID, output ID, a zero reserved word, UTF-8 byte count and name, followed by a
three-byte opaque suffix. The decoder validates the exact record length and both
object references before allowing structural edits. Imported definitions appear
in the desktop Transform menu. The single-page writer preserves the opaque
record, remaps both IDs when objects are compacted, and removes the definition
when either endpoint is deleted. Newly defined, renamed, reordered, or removed
entries remain document-session state; already applied mapped points are
ordinary document objects and persist normally.

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

The supplied Native fixture `未实现的系统功能/自定义变换.gsp` now opens as a
complete editable 10-object document and restores `变换 1` (point 4 to point 8)
in the menu. A headless application creates native mapped object 11 with carried
graph `[5, 6, 7, 8]`, then saves and reopens as a complete document. Deleting
the output removes both its dependent sampled mapping and the stale definition;
the resulting eight-object file also saves and reopens without a partial view.
An Xvfb desktop run confirms the Chinese menu label renders through the system
CJK fallback. Sketchpad 5 also opens and resaves the imported-definition plus
applied-point result without changing any of its 3,140 bytes (SHA-256
`ac46264d76bfaabd9e9e63c3990e164a40626c66a2c48e5f87d1d5a3ebfed10a`).

Across the 1,599-file corpus, recognizing the strictly validated metadata lowers
unrecognized-record restrictions from 164 to 31 and total partial views from
538 to 405. All 285,703 object slots still decode, all 5,356 embedded-tool slots
remain decoded, and all 1,599 original files retain byte-exact round trips.

Workspace formatting, Clippy with warnings denied, both release Nix packages,
the full corpus scan, the Xvfb menu smoke, and the targeted Native cross-save
all pass for this change. No duplicate fixture-specific test cases were added.
