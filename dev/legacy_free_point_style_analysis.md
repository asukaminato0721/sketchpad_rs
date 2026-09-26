# Legacy free-point style fields

The native Clock(2) free point has a 16-byte header:
`000000000000010004000100ff000020`. Its observed layout is code 0, class 1,
subtype 4, and the 16-bit word at offset 10 is 1. Reading byte 11 as the modern
point-size byte incorrectly classified this point as Dot. Native copy/paste
upgrades it to a 28-byte header whose explicit modern size byte is 1 (Small).

Controlled probes were used to avoid inventing an offset or arithmetic mapping.
Changing only byte 11 to 1 or 2 in a mixed-value control file produced a native
format error. Separate files changing just the first point to byte 11 equal to 1
and to 2 were then each independently rejected with native format-error dialogs
(`/tmp/legacy_free_high_style_{1,2}_native_open_failed.png`). Changing the word at offset 10 to 0/1/2/3, then separately using
only positive 1/2/3 values, also produced a native format error. These failed
files do not prove a size mapping. Their captures remain at
`/tmp/legacy_point_{styles,low_styles,positive_styles}_native_copy_open_failed.png`.

The successful control preserves the structural word at offset 10 and changes
only the fourth RGB-adjacent byte at offset 15 to 00, 20, 40, and 60 for the four
points. Native displays all four alike. Native copy/paste, followed by deselecting
on empty space and saving, upgrades both original and pasted free points to
28-byte headers with size byte 1. All eight points remain Small; their byte 15
becomes zero while RGB bytes remain unchanged. Thus the tested old byte-15 bits
are not mapped to point size either. The native output is 2660 bytes. It retains
four original and four pasted points, the original polygon, and its pasted copy.

The native before-save and after-copy captures are
`/tmp/legacy_point_packed_styles_native_copy_before_save.png` and
`/tmp/legacy_point_packed_styles_native_copy_after_setup.png`. They show unchanged
point footprints at all corners, and the copied geometry at the native paste
offset. `legacy_free_point_style_opaque_original.gsp` retains the controlled
short-header input; `legacy_free_point_style_opaque_copied_native.gsp` retains
the native converted/copied result. This is a conversion proof, so the source
and converted output are intentionally not asserted byte-identical.

The bounded style predicate is private to `style_records.rs`: exactly 16 bytes,
code 0, class 1/subtype 4, and bytes 10–11 equal `01 00`. These observed legacy
free-point headers have implicit Small size. The getter returns Small regardless
of the opaque byte 15. Setting Small preserves the complete header. Setting Dot,
Medium, or Large fails before mutation because no editable short-header size
field was established. Whole-document conversion to modern headers can provide
an explicit size field; this setter does not expand a record or rewrite unrelated
legacy metadata. All opaque bits, colors, and flags remain untouched.

Other point layouts retain their original interpretation. In particular a
16-byte prefix with modern flag byte 10 equal to 21 still reads/writes explicit
byte 11, just like the corresponding 28-byte modern header. That compatibility
regression checks the existing interpretation; no native short modern fixture
was discovered in the corpus, and it is not presented as new native evidence.
The corpus scan found the exact legacy free-point family only in Clock(2), one
object with the word `01 00`.

Regression tests compare legacy input styles with all eight native converted
headers, retain RGB and opaque fields, reject unsupported size setters atomically,
and preserve every other byte when changing a modern short-header size. Fresh
modern clipboard goldens must now use Small1 for copied legacy free points;
modern point fixtures are unaffected.
