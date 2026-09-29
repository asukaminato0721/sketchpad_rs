# Sampled custom-transformed pictures (code 96)

## Evidence

The corpus contains 18 drawing groups with construction code 96, class 6,
subtype 91 in four documents. In each construction a hidden source picture is
followed by a visible code-96 object. The references have the stable form
`[output point, source picture, seed point, carried...]`; the carried list ends
at the output point and passes the same bounded custom-point-mapping validation
used by native mapped objects.

Every admitted group has exactly these records:

- `07d0:28` — code 96, class 6, subtype 91
- `07d2:4+4*n` — counted references
- `07d3:12` — code 96 followed by the `f64` sampling resolution (`0.6` in all
  observed groups)
- `0902:40` — native sampling range/cache
- `07d7:0`

The range record has domain `0.0..1.0`, a sample budget at bytes 16–19, flags
`0x00010004`, a required word `1` at bytes 32–35, and a zero tail. Bytes 24–31
remain opaque and are retained exactly. Groups outside this schema remain
unsupported.

## Model and evaluation

`SampledImageDefinition` retains the source picture, mapping seed/output,
carried dependency DAG, resolution, and complete range payload. Admission
requires earlier live same-page references, one ordinary picture source,
point seed/output roles, a unique carried list ending in the output, and the
existing supported custom-point evaluator boundary.

Rendering creates a bounded UV grid over the source pixels, substitutes each
grid point for the mapping seed, and evaluates the mapped output in a cloned
scoped document. Undefined samples suppress only their adjacent triangles.
The desktop canvas paints the resulting textured mesh in native picture order;
picking tests the same triangles. SVG export emits a shared image definition
and affine/clipped textured triangles, so the nonlinear mapping is retained
rather than flattened to a bounding box.

The writer emits the exact five-record native group and preserves the opaque
range bytes. Dependencies and copy remapping treat the sampled picture as a
constructed picture, not as another curve family. No editor command for
authoring a new custom-transformed picture is exposed yet.

## Verification

The real desktop editor opened the native `折扇2（钮炳坤老师）.gsp` file under
Xvfb and rendered the visible character pictures on the fan leaves while their
source images remained hidden. The same document exported a sampled SVG mesh.

Corpus coverage changed as follows:

- unsupported slots: **769 → 751**
- fully decoded documents: **1,510 → 1,513**
- documents with unsupported objects: **89 → 86**
- drawing-complete documents: **1,551 → 1,554**
- partial views: **800 → 800**
- byte-preserving originals: **1,599 / 1,599**

Embedded-tool coverage remains **4,766 / 5,356**. No standalone test case was
added; existing tests, strict clippy, corpus coverage, byte round trips, SVG
export, and the Xvfb desktop run provide the checkpoint gates.
