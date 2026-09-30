# Attached pictures in embedded-tool invocation

Audited on 2026-09-30. Three embedded prototypes have the same 69-object
construction and differ only in their containing documents:

- `Samples/个人专栏/钮炳坤作品/500位PI螺旋.gsp`, tool 0;
- `Samples/个人专栏/钮炳坤作品/根号2(钮炳坤).gsp`, tool 0;
- `Samples/热研系列/迭代系列/500位PI螺旋.gsp`, tool 0.

Each prototype declares 39 inputs: 35 generic Numeric values followed by four
Points. The first 68 local objects were already decoded. Local object 69 was
the final unsupported slot, a code-100 class-16/subtype-78 picture attached to
three earlier constructed points.

## Native record evidence

All three local object-69 groups use the same bounded record family:

1. `07d0`, code 100 and class 16/subtype 78;
2. `07d2`, references `[66, 68, 67]`;
3. `090c`, dimensions `[width, height, width, height]`;
4. `07d5`, the exact hidden internal `[1]` label;
5. `1f44`, resource index 0;
6. `07d7`.

The 28-byte hidden-label payload is byte-for-byte the one already established
for attached code-99 pictures:

```text
06 00 00 00 00 00 08 00 05 00 00 00 00 00
98 6e 12 c1 51 5c 61 41 03 00 5b 31 5d 00
```

The decoder now recognizes that exact auxiliary record for code 99 and code
100. A successfully decoded PNG catalog also makes its corresponding `0514`
resource structurally understood. The isolated prototype remains
`partial_view` because imported resources are read-only, but its structure is
safe and contains no unsupported objects. This distinction permits immutable
tool invocation without presenting the prototype as generally editable.

## Bounded writer and invocation

The image writer can now emit the proved code-100 parallelogram family from a
PNG and three positive object references. It writes the exact header, repeated
dimensions, hidden label, resource-zero reference, and one `0514` PNG resource.
Other attached placements remain rejected. Document validation still permits
only one picture in a writable document, so this does not claim general
multi-picture or resource-table authoring.

Tool invocation copies numeric caption fonts from the prototype. The relevant
visible numeric sequence uses normal Times New Roman size 16: local object 50
has caption anchor y=415 and local object 51 follows at y=436, establishing a
21-pixel placement step. Local object 56 is a flat-caption `NumericAlias`, so it
uses the same caption-layout path. Hidden numeric objects without captions do
not invent a font override.

A synthetic destination supplied four points and reused one numeric value for
the 35 generic Numeric inputs. Invocation created the expected 30 dependent
objects, including the final parallelogram-attached image, then encoded and
reparsed all 36 destination objects exactly. The generated file is 8,596 bytes.

Native Sketchpad 5 opened the port-authored file under Xvfb/Wine and visibly
rendered the four points, numeric captions, and attached PNG. Ctrl+S preserved
the file byte-for-byte with SHA-256
`a811318060ca4b96b496b34845f02e3a7fda2d05d51d95fdb61e56ad4500e063`.

## Measured recovery

The three code-100 roots are now decoded and invocable. Prototype coverage
rises **5,070→5,073/5,356**, unsupported slots fall **286→283**, and documents
with all object kinds decoded rise **1,572→1,575/1,599**. Complete isolated
views remain 49 because these image-bearing prototypes intentionally retain
their read-only-image partial status. All 1,599 drawings remain complete, and
all 1,599 original files remain byte-exact. No standalone test case was added.
Full scan: `/tmp/tool_code100_coverage.txt`.
