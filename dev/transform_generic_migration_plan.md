# Generic transform representation cutover

The next representation is `Kind::Transformed(TransformDefinition)`. Keep the
existing 60 variants temporarily for constructors and regression expectations,
but normalize successful imports and newly appended objects into the generic
variant once consumers support it. This changes stored scene objects rather
than adding another evaluation-only adapter. Remove compatibility variants only
after their construction and inspection sites have migrated.

`TransformDefinition` owns semantic operation, source, element, native flavor,
literal roles, and ordered references. `Object` and the original source bytes
continue to own header/constraint offsets, native caches, labels, visibility,
styles, signed polygon ranks, auxiliary records, and unknown presentation data.
Do not reconstruct imported groups from authoring templates on this cutover.

## Decoder boundary

The transform branches are contiguous in `document/decode.rs`, between code21
translation and code27 rotation, before the ordinary code1 midpoint branch.
They currently construct all 60 legacy variants. Preserve their exact guards,
then construct a shared definition for each accepted result. Reversible first
step: extract that match block into an import helper that returns a legacy kind;
after the generic variant exists, normalize its successful result at the helper
boundary. Returning `Unsupported(code)` for a recognized rejected transform must
remain distinct from returning no result for an ordinary construction code.

Suggested helper signature:

```rust
fn decode_transform(
    code: u16,
    header: &[u8],
    references: &[usize],
    constraint: Option<&[u8]>,
    objects: &[Object],
) -> Option<Kind>
```

Here `None` means a non-transform code; `Some(Unsupported(code))` means the
transform layout or prerequisites failed. Existing record/body validation,
page scope, deleted-object checks, and metadata decoding remain outside this
helper. Do not substitute authoring validation for the importer guards.

| Native code | Flavor | Important retained import behavior |
| --- | --- | --- |
| 16 | Vector | Source family classifiers; point start/end; polygon subtype count. |
| 17 | Fixed Cartesian | 52-byte constraint; two unit tags; finite caches; fixed centimeter cache tolerance. |
| 18/19 | Fixed horizontal/vertical | 28-byte constraint and tail; marked-distance category; base shape restrictions. |
| 20 | Marked Cartesian | Absent constraint; marked-distance category for both coordinates. |
| 21 | Fixed polar | 60-byte constraint; finite trigonometric caches; restricted source families; polygon cache conversion below. |
| 24 | Marked polar | 36-byte constraint; angle cache tolerance and unit tag; marked-distance category. |
| 27 | Fixed rotation | 36-byte constraint; trigonometric cache tolerance; subtype/class agreement; verified ray/line modes. |
| 28 | Point angle rotation | Five ordered references; point prerequisites; base segment/circle/polygon restrictions. |
| 29 | Numeric rotation | Absent constraint; degree-valued numeric operand; class/subtype agreement. |
| 30 | Fixed dilation | 12-byte constraint; finite factor; verified ray/line modes; native base polygon restriction. |
| 32 | Point ratio dilation | Five ordered references; point prerequisites; flavor-specific source restrictions. |
| 33 | Numeric dilation | Absent constraint; scalar operand; class/subtype agreement. |
| 34 | Reflection | Straight mirror; flavor-specific source family and polygon subtype checks. |

Code21 polygon import has a native cache form identified by the exact
`payload[52..60]` signature `[0x64, 0xe4, 0x83, 0x01, 0, 0, 0, 0]`.
For nonzero distance its semantic unit scale is `payload32 / distance`, rather
than raw `payload32`. Preserve that recovery and the untouched cache bytes.
Fixed Cartesian and mixed literals currently recover centimeters at
37.79527559055118 pixels per centimeter. Their existing tolerance checks must
survive unchanged; this migration does not broaden accepted scale layouts.

## Ownership and dependent changes

| Area | Required next change | Current boundary |
| --- | --- | --- |
| Root: `document.rs`, `kind.rs` | Add generic variant; normalize append; recognize generic definitions in classifiers, remapping, dependencies, default styles. | Do not edit the root-owned enum during preparatory work. |
| Import/wire: `decode.rs`, `transform_model.rs` | Extract native guard block; normalize accepted kinds; replace constructors with shared operation construction when legacy variants are removed. | Preserve raw metadata outside the definition. |
| Import/wire: `encode.rs`, `encode_patches.rs` | Shared transform writer already works through definition; replace remaining locus/cache and transformed-polygon marker matches with exact flavor/element predicates. | Preserve code58 Segment-or-VectorSegment capability. |
| Evaluation: `geometry.rs`, `transform_evaluator.rs` | Shared evaluation already consumes definitions; remove adapter-only fallback matches after generic storage is active. | Keep source-first undefined propagation and straight modes. |
| Evaluation: `iteration.rs` | Migrate point-image whitelist, two-translation branching whitelist, mapped dependency whitelist, and point-step evaluation. | Preserve each native iteration family's distinct substitutions and carried source restrictions. |
| Root/editor: `document/transforms.rs`, `editor/transforms.rs` | Build shared definitions instead of the 60 constructors. | Keep command capability guards and live operand references. |
| Root/presentation: `render.rs`, `editor/picking.rs` | Replace circle/polygon product matches with existing element classifiers or generic element guards. | Keep polygon fill ordering and picking geometry. |
| Probe/tests | Replace product matches with semantic definition assertions as variants are removed; retain native byte fixtures. | Keep supported/unsupported chain and unit regressions. |

Do not normalize only after the whole document has decoded. Later groups inspect
earlier objects while decoding, notably mapped iteration carried bindings that
currently allow only base segments and vector-translated segments. Those guards
must recognize the exact generic flavor before import storage changes.

The combined cutover gate must cover authored/imported native goldens, copies,
multi-page remapping, signed polygon order, live numeric transforms, unsupported
source families, iteration substitution, zero-scale ancestors, labels/styles,
rendering/picking, and corpus byte preservation. Existing native fixtures remain
the evidence for wire behavior; representation changes add no native family.

The preparatory presentation changes are now in place: circle/polygon mesh
dispatch and polygon picking use the existing element classifiers, and the
moved-marker patch uses the exact transformed-polygon element predicate. The
primitive families retain their original handling; base polygons do not acquire
the transformed-polygon marker rule. No decoder or stored-object normalization
has been enabled by these preparatory edits.

## Implemented storage cutover

The planned storage cutover is implemented: Kind has one Transformed variant,
and all 60 former constructors are now compatibility functions creating that
variant directly. The import guard block stays unchanged; earlier objects are
already generic while the next group decodes. All listed consumer pattern
matches have migrated to exact shared-definition predicates. Ordered dependency
and reference remapping use semantic fields, including repeated references.
Native object metadata remains outside the definition. The compact constructor
macros share each native flavor expression across its supported elements.
Named constructor compatibility can be removed in a future public API change;
it no longer introduces distinct stored variants or operation-specific runtime
branches. The combined all-targets gate passes 806 tests. The corpus gate preserves all
1599 documents byte-identically and retains the same decoding coverage.
