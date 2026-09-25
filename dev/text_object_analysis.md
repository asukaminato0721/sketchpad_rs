# Free text object encoding and editor integration

The original Sketchpad 5 Text tool creates a free caption after a double click on blank canvas. Original fixtures live in `tests/fixtures/text_object_original_*.gsp`; generated and dragged files use the matching `text_object_generated_*.gsp` names. `dev/text_object_wine_create.sh` reproduces the original creation step under the shared Wine lock.

The object group is `0x07d0` (code 0, text subtype 0x13), `0x08fc` rich text, `0x07d8` style (`00 00`), `0x0898` four `i16` rectangle coordinates, and `0x07d7`. The rich-text payload has magic `0x08fc` at byte 0, a two-byte-aligned inner size at byte 4, flags 4 at byte 8, an `i16` anchor at 12/14, markup byte count at 16, and markup at 18. The outer record payload rounds to four bytes. Literal lines use `<VL<T{HEX_BYTE_COUNT}x{RAW_UTF8}>>`; additional lines add more `<T...>` runs before the final `>`. The count is UTF-8 bytes. Angle brackets in the literal are allowed because the run length gives their boundary.

The encoder matches original `A`, `Hello`, and multiline groups byte for byte when given the same rectangle. Generated Hello, Unicode, literal angle brackets, multiline, and blank-line sketches opened in the original app. A generated Hello was dragged in the original app; its anchor and rectangle moved together and a selection record was added. The app also wrote a nonzero byte in one unused inner padding slot. The parser accepts this padding and preserves it in an unchanged file; editing the literal rebuilds the validated canonical group.

The document model recognizes literal `<VL<T...>>` runs and the direct `<T...>`
runs Sketchpad writes after formatting a text object. It validates the
`0x07d8` style index against the document's `0x04b3` font-style records. When
editing or moving saved text, it patches only the rich-text and rectangle
records, keeping the style index and the other objects' runs intact. Native
two-text probes with either caption made nonbold import as editable documents;
edits to each caption opened and saved byte-identically in Sketchpad 5.
Native Text tool probes selected the first two characters of `Hello` and applied
Bold, Italic, Underline, and Bold+Italic. They produced a horizontal rich-text
group inside the vertical wrapper, for example
`<VL<H<SW1S2<T2xHe>><T3xllo>>>`. `W1` disables Bold for that run, `S2`
enables Italic, and `U2` enables Underline; the second run inherits the object
style. The port parses these style attributes with bounds checks, displays the
separate runs on the desktop and in SVG, and preserves them when moving or
copying a caption. A moved mixed-format caption and a copied one both opened
and saved byte-identically in Sketchpad 5. The copied file is
`tests/fixtures/text_mixed_copied_generated.gsp`.

The editor sidebar now lets a user select characters in a caption and toggle
Bold, Italic, or Underline for that range. It writes the same `<H><S...><T...>`
structure; a port-generated caption with nonbold italic `He` and bold `llo`
opened visibly and saved byte-identically in Sketchpad 5. Its fixture is
`tests/fixtures/text_range_styled_generated.gsp`. Native edit probes showed
that an insertion at a run boundary inherits the character on its left, while
replacement text inherits the first removed character's style. The port now
maps unchanged characters through text edits and applies those rules to new
characters. Generated boundary insertion, replacement, and newline edits all
opened visibly and saved byte-identically in Sketchpad 5; their files are
`text_mixed_insert_generated.gsp`, `text_mixed_replace_generated.gsp`, and
`text_mixed_multiline_generated.gsp` under `tests/fixtures`. The insertion and
replacement run markup matches native-created files byte for byte. Ambiguous
edits to repeated characters may choose a different style mapping because the
sidebar applies a completed plain-text edit rather than each keystroke.
The native Text Palette wrote a selected 36-point `He` as
`<VL<H<S#24<T2xHe>><T3xllo>>>`; `#24` is the hexadecimal size code. The port
decodes and writes this size on individual runs. Applying the next verified
size step to the native file wrote `#30` for 48 points. Sketchpad 5 displayed
the port-generated file and saved it byte-identically. The corresponding
fixtures are `text_mixed_size36_he_original.gsp` and
`text_range_size48_generated.gsp`.
Applying TeX Gyre Adventor to the selected `He` in the native Text Palette
created a second `0x04b1` family record and wrote
`<VL<H<SP1<T2xHe>><T3xllo>>>`. `P1` refers to family index 1 while the object
style still supplies Tahoma to `llo`. The port now resolves this per-run font
index in the desktop canvas and SVG, preserves it through edits, and can add a
new family record when the sidebar formats a character range. An inserted
character in the native mixed-family fixture and a port-generated selected-font
caption each opened and saved byte-identically in Sketchpad 5. Their fixtures
are `text_mixed_adventor_he_original.gsp`,
`text_mixed_adventor_insert_generated.gsp`, and
`text_range_adventor_generated.gsp`. A combined size and family run
`<S#24P1...>` also opened and saved byte-identically in Sketchpad 5; its
fixture is `text_range_family_size_generated.gsp`.
Expression markup remains unsupported. Source text and point labels are
preserved byte for byte on an unchanged save. The saved sketch is reparsed and
compared before atomic replacement. Labels and text participate in undo/redo
through object snapshots.

`tests/text_object_core_replay.txt` constructs a point label and text, renames, hides/shows, changes text to `Hello\n猫`, moves it, saves and reopens. The port output passed `gsp_probe verify`; the original Sketchpad 5 opened and saved it. The screenshot `/tmp/sketchpad_text_core_wine_before_save.png` shows `Hello` and `Cat`. The supplied Wine font lacks a Chinese glyph, so `猫` appears as a missing-glyph square in that screenshot, just as the original-created Unicode fixture does. SVG export contains the UTF-8 text and the point label.
