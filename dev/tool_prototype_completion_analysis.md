# Complete embedded-tool prototype decoding

Audited 2026-09-30 against all 1,599 corpus documents and Native Sketchpad 5
under Xvfb/Wine.

## Final prototype families

The last 27 unsupported slots belonged to five prototype-only typing families:

- six numeric dilations use a declared generic Numeric given as their ratio;
  five vector translations become ready after those points decode;
- two sampled mappings use an abstract point-on-path seed, with one also
  depending on a generic Numeric given outside the substitution;
- a polygon-surface tool uses literal points on a declared Path, then derives a
  fixed dilation, rotation, point ratio, calculation, and two colorized outputs;
- an angular Parameter given drives a point Parameter Locus with three
  unspecified output points on that curve;
- a ruler tool intersects a constructed line with a declared Path.

Each admission remains confined to isolated tool decoding. Invocation replaces
every abstract input before ordinary document validation. Generic Numeric
dilations must become scalar numeric operations; sampled path points become
ordinary constrained points; a generic colorized Path specializes to the
bound Segment, Ray, Line, Circle, Arc, or Polygon; and the polymorphic
intersection specializes to either straight/straight or straight/circle/arc.
Unsupported polygon intersection bindings are rejected transactionally rather
than assigned invented semantics.

The surface tool's code-123 points preserve their literal parameters while the
bound Path remains abstract. Its colorized point and generic Path use the same
ordinary live color evaluator after specialization. The rolling tool's angle
Parameter retains the established radians sampling domain, and its three
unspecified curve points receive stored ordinary curve parameters at invocation.

## Runtime and Native evidence

Invoking `2.实点(x,y,z)` with four points and scalar values 2, 3, and 4 creates
eight outputs. The 15-object, 3,196-byte result reopens non-partially and
re-encodes exactly. Native Sketchpad visibly renders the three constructed
segments, and Ctrl+S preserves SHA-256
`7789d4f890b65ffba6bef452ff93fc84d153a845fca561500fc70c73e3e65606`.

Invoking `直尺画线工具` on a real Circle and Point creates four outputs. The
8-object, 2,404-byte result reopens non-partially and re-encodes exactly.
Native Sketchpad opens and renders it, and Ctrl+S preserves SHA-256
`8e3a0b8f7459c64a2227ac96f04f8ffc4f611e19bb060d6bff07757adba362c4`.

The two compass sampled-mapping tools and polygon-surface tool now decode
completely. Subsequent structural work resolved their runtime caption placement
through Native's stored presentation suppression and anchor reuse; no guessed
font metrics were introduced. See
[the structural follow-up](tool_prototype_structure_analysis.md).

## Corpus result

Prototype coverage rises **5,329→5,356/5,356** and unsupported slots fall
**27→0**. Fully decoded documents rise **1,593→1,599/1,599**. Complete isolated
tool views rise **78→86/103**; the other 17 are structurally partial for reasons
other than unsupported object kinds. All 280,347 drawing slots remain decoded,
all 285,703 total object slots are recognized, and all 1,599 originals
round-trip byte-exactly with zero failures.

Subsequent structural work raises the current isolated-view result to 96
complete and seven image-read-only views; all seven bounded picture prototypes
can now be invoked. This proves complete corpus object decoding and the bounded
tool frontier, not complete editing or command parity. No standalone test case
was added or removed. Full scan:
`/tmp/tool_prototype_complete_coverage.log`.
