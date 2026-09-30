# External tool placeholders in mapped iteration

Snapshot: 2026-09-30.

## Observed structures

The largest ready prototype root was code 89 in
`任意边数凸多边形在直线上滚动工具（李章博）.gsp`, tool 0, local object 68.
It stores three scalar seeds, one branch, 27 carried constructions, and a live
depth value. Its carried graph ultimately depends on two custom-tool
placeholders:

- local 2 is a polygon input;
- local 6 is a point chosen on the tool's constructed ray.

Neither placeholder is stored as a seed, branch, or carried result. They are
invariant predecessors. During invocation the polygon is replaced by the bound
destination polygon and the path point is materialized as an ordinary point on
the remapped path. The mapped evaluator then freezes both at its existing
external-input boundary.

The same distinction occurs in three fixed-depth code-76 mappings in
`游标卡尺2.gsp`, tool 0, locals 93, 102, and 108.

## Admission rule

The mapped validator continues to reject generic tool placeholders as mapped
value constructions. It recognizes only polygon inputs and tool points on a
path, and only while traversing an ID that is absent from all three active
mapping sets: seeds, branches, and carried values. Dependencies of the external
placeholder are still checked, so this does not allow an unresolved path or an
out-of-order construction.

This keeps placeholders out of substitution state and output bindings while
matching the evaluator's existing invariant-input behavior.

## Corpus result

`gsp_probe coverage ../gsp-rs/tests` reports:

- 1,599/1,599 byte-exact round trips, zero parse failures;
- prototype slots decoded: 5,073→5,110 of 5,356;
- unsupported prototype slots: 283→246;
- direct mapped descriptors restored: three code 76 and three code 89;
- dependent slots restored: seven code 77, ten code 48, three code 88, six
  code 95, three code 21, and one each of codes 41 and 94.

All drawing kinds remain decoded in every corpus document. The all-kind
document count remains 1,575 and complete isolated tools remain 49 because the
affected prototypes still contain unrelated unsupported constructions.
