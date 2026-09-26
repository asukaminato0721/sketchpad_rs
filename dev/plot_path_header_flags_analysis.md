# Native plot-point header flags

This audit is bounded to 28-byte class-1 code-95 and code-123 point headers.
The first word stores a 16-bit operation followed by 16-bit flags. A scene-only
scan excluding embedded tool canvases found code-95 flags 0/1/2/3/4 in
596/1214/24/521/84 groups and code-123 flags in 38/113/19/255/2 groups.
Flag 4 remains unproved. Tracing is independently stored in header byte 4.

The controlled 20-object fixture uses an ordinary horizontal segment and four
scalar parameters. It contains each flag 0–3 once for code 95 and once for
code 123. Native opening and saving preserves the 4,414-byte fixture exactly.
Flag 0 points and captions display. Flag 1 points are hidden. Flag 2 and 3
points and captions do not display.

Two visible marker segments were then attached to the flag-2 points. Native
markers display, establishing that both points have defined geometry. An actual
native mouse drag moves the host endpoint from (290,190) to (330,190). The
code-95 marker endpoint changes from (190,190) to (205,190), and code-123
from (270,190) to (305,190), as their affine parameters require. Flags 2 and 3
remain unchanged and the point glyphs remain absent. This is persistent
presentation suppression, not an undefined-position or stale-cache inference.
The native-edited fixture has 22 objects and 4,514 bytes.

Native Show All Hidden clears only ordinary flag 1 to flag 0. It leaves
both flag 2 and flag 3 exactly unchanged. The 4,514-byte result retains all
22 objects. Thus folding flag 2 into the editable visibility bit would lose
native behavior for Show All Hidden.

Native Select All, Copy, Paste creates a 40-object, 6,780-byte document.
The suppressed code-95 source 15 becomes copied point 36, referenced by
copied marker 39; the suppressed literal code-123 source 19 becomes copied
point 38, referenced by copied marker 40. Both new points have ordinary flag 1,
while source flags 2 and 3 remain unchanged. Native Show All Hidden then changes
new points 36 and 38 to flag 0 and displays their glyphs and captions. Original
flag-2 and flag-3 points remain suppressed. Source suppression therefore does
not require an independent clipboard field: copying converts these dependencies
to ordinary hidden points.

An actual native Select Parents command on marker 21 highlights its visible
FreePoint parent. Display > Hide changes that parent from flag 0 to flag 1,
confirmed by an intermediate Ctrl+S capture. Its suppressed code-95 parent 15
remains flag 2. Subsequent Display > Show All Hidden restores the free parent
and ordinary hidden plot points while retaining flags 2 and 3. The invisible
point does not acquire a selection glyph, so this control does not assert
whether it belongs to the native internal selection; it establishes the actual
parent-selection and hide/show command behavior.

The parser now separates the low 16-bit operation from the flags and accepts
only the proved flags 0–3. Flag 4 and other bits remain rejected. A bounded
source-header predicate suppresses display only for known code-95/code-123
points with bit 2. Their live geometry and editable bit-0 visibility remain
independent. Ordinary hide/show commands skip suppressed source points, and
clipboard capture turns suppression into ordinary hidden visibility. Existing
source bytes retain the opaque suppression bit; new copied points use flag 1
and can subsequently be shown.

Regressions cover the entire source becoming known, all eight point positions,
byte-preserving source save, hide/show with undo/redo, no-op explicit visibility
commands on suppressed source points, source vertex and scalar value edits,
copy dependency remapping, flag-1 copied headers, copied Show All recovery, and
save/reopen. Unknown flag 4/5/high-bit layouts remain rejected. Actual authored
writer checks all exited successfully before byte comparison: source-edited
4,510 bytes, copied 5,596 bytes, and copied Show All Hidden 5,596 bytes. Each
opened and saved byte-identically in Native. The public-API regression compares
all three results against retained native-verified goldens. Native captures
show source suppression surviving the dependency edits, copied points initially
hidden, and both copied glyphs and captions after Show All Hidden.
