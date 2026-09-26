# Pasting into legacy documents

The original 1,492-byte legacy polygon fixture contains four free points and
one polygon with 16-byte headers. Ordinary modern additions use 28-byte
headers. Native Sketchpad upgrades the complete source document when pasting;
saving a mixture of the old and modern headers is not a valid substitute.

A controlled conversion keeps every original record except the object headers
and the version word. Record 0x0384 changes its first u16 from 4 to 5; the outer
signature remains GSP4. Page preferences, font/style tables, labels, compatibility
messages, window fields, and application-version record remain intact. Native
Sketchpad opens and saves this bounded conversion byte-identically. Replacing
the compatibility prefix or Arial font with modern stock metadata is unnecessary.

The proved short free-point header has code 0, class 1, subtype 4, and structural
word 1 at offsets 10–11. It expands to 28 bytes, sets bit 0x20 at offset 10,
stores implicit Small as byte 11 = 1, and clears old byte 15. The short polygon
header has code 8, class 4, subtype at least 52, and the same structural word.
Its modern tail stores the proved opacity 0xbfff and drawing rank 0. Standard
modern cache/header tails supply the remaining fields. Other short layouts
require separate native evidence before conversion.

`Document::paste` performs this upgrade on its candidate before appending the
clipboard objects. A structural checkpoint retains the old source bytes and
source-object count, so undo restores the original short headers and metadata;
redo restores the modern document. Prior edits are encoded before upgrading.
The converted graph is reparsed and checked against the expected objects and
styles, with only the old polygon's absent rank becoming explicit zero. Failed
conversion leaves the caller and its undo history unchanged.

The actual ordinary writer output, copying polygon 5 and its four dependencies
within the same document, is retained as
`legacy_polygon_same_document_native_verified.gsp` (2,072 bytes). Its native
process completed successfully before a byte comparison; the screenshot shows
two red 75% polygons with Small red vertices, the copied set displaced by
(20,20). The capture prefix is `/tmp/legacy_polygon_same_document`.

Regression tests assert the native writer golden, full header conversion,
point styles, save/reopen, undo/redo, preservation of prior color edits across
two pastes, and atomic failure on an unproved legacy header. Generate the
actual writer probe with `SKETCHPAD_WRITE_LEGACY_PASTE_FIXTURE=1 cargo test
legacy_paste`; it writes `/tmp/legacy_polygon_same_document_port.gsp`.

This conversion covers the proved free-point/polygon layouts and primitive
strokes documented below. Other old object families and command parity remain
pending.

## Style edits and additional primitive strokes

The converter also covers independently proved 16-byte Segment (code 2, class 2,
subtype 14), Circle (code 3, class 3, subtype 28), Ray (code 64, class 2, subtype 13),
and PerpendicularLine (code 5, class 2, subtype 47) headers. Their old pen enum
is not a packed width/pattern: 0 means ThinDashed, 1 ThinSolid, and 2 MediumSolid.
The exact modern byte 10 mappings are 0x11, 0x21, and 0x22, with modern byte 11=1.
See the separate legacy stroke analysis for the source/native-copy controls.
Unproved old enum values and other old families remain excluded.

The public style API checks whether the existing short header can represent
an edit. Color and representable old pen-style changes preserve the short
records. A new point size or stroke combination upgrades the whole supported
legacy document before applying the style. A structural checkpoint preserves
its original source for undo. Pending deletions are compacted during upgrade;
selected objects and style changes follow their remapped surviving IDs. The
candidate is verified before replacing the caller, so a failed edit leaves
both document state and undo history intact.

Native Display > Point Style > Medium on point 1 of the legacy polygon fixture
upgrades all five headers to 28 bytes. The 2,316-byte native-created file is
`legacy_point_medium_native_ui_verified.gsp`; the corresponding ordinary Rust
writer output is `legacy_point_medium_upgraded_native_verified.gsp` (1,552 bytes).
The native UI replaces stock document metadata/fonts; the Rust writer preserves
the original metadata and fonts. Native Sketchpad opens and saves the actual
Rust output byte-identically, with the same Medium first point, remaining Small
points, and 75% polygon. Tests compare effective styles and ranks against the
native UI fixture, assert the exact writer golden, and exercise one-step undo.
The native UI setup script is `dev/legacy_point_style_wine_setup.sh`; capture
prefixes are `/tmp/legacy_point_medium_{native_ui,upgraded}`.

The actual same-document stroke clipboard writers both passed native opening
and byte-identical saving at 2,696 bytes: mixed original pen styles and all-dashed
styles. The mixed output is `legacy_stroke_paste_upgraded_native_verified.gsp`;
the dashed output is `legacy_dashed_paste_upgraded_native_verified.gsp`.
The latter was refreshed by two Ctrl+F6 MDI switches before inspecting the image;
all circles, segments, rays and perpendiculars render with Thin Dashed strokes
and their copies at the 20-unit displacement. Capture prefix:
`/tmp/legacy_dashed_paste_visual`. An initial black screen and a later incomplete
File Open dialog were environment/UI observation failures, with no native format
alert; the same inputs subsequently opened successfully. No decoder widening or
record changes were made in response to those failures.

Changing the legacy ray to Thin Dotted requires modern storage; the ordinary
1,864-byte writer opens and saves byte-identically as
`legacy_stroke_dotted_upgraded_native_verified.gsp`. A refreshed native
screenshot confirms the dotted ray alongside the original
solid segments and perpendicular, Medium Solid left circle, Thin Solid right
circle, and Small red points. Capture prefix:
`/tmp/legacy_stroke_dotted_visual`. The mixed clipboard geometry was also
verified after MDI refresh at `/tmp/legacy_stroke_mixed_visual`. Regression tests
assert the exact bytes of both clipboard writers and the dotted edit.


A full read-only corpus audit found one document with 16-byte headers:
`../gsp-rs/tests/Samples/未分类档/时钟 (2).gsp`; the other 1,598 documents use
version 5 records. That legacy clock also contains constrained points,
transformed points and segments, numeric captions, and other short-header
families beyond the proved converter. Its broader conversion remains guarded.
The focused conversion fixtures establish the supported layouts independently;
they do not establish editability of the entire legacy clock.
