# Circular-segment boundary point

## Scope

The corpus contains one drawing object with native code 15, header class 1 /
subtype 10, and constraint subtype 69. Its single reference is a live code-83
circular-segment boundary. The implementation is deliberately limited to that
observed layout and host relationship; unrelated code-15 profiles remain
unsupported.

Fixture:

`../gsp-rs/tests/fixtures/未实现的系统功能/弓形周界动点.gsp`

## Native layout and geometry

Object 7 has the standard 28-byte point header, exactly one reference, and the
12-byte constraint payload:

```text
0f 00 45 00 11 4f 9d 5b 1f b4 ee 3f
```

The leading words are code 15 and subtype 69. The trailing little-endian `f64`
is `0.9594876088471854`. The host is object 6, a circular-segment boundary over
the preceding center arc.

Subtype 69 uses a normalized physical-perimeter parameter. It traverses the arc
from start to end, then the closing chord from end back to start. In the fixture,
the late parameter therefore places P on the chord close to A; the port render
matches that construction. Projection for dragging compares the analytic arc
and chord candidates rather than a sampled display polygon.

## Reader, editor, and writer boundary

The reader requires the exact observed header, record lengths, constraint
prefix, one earlier same-page live circular-segment host, and a finite unit
parameter. Position evaluation, selection, movement, measurements, loci, and
animation use the same live boundary model.

Both single-page and multi-page writers emit subtype 69 for newly represented
objects and patch only the stored parameter when editing an imported object.
The original fixture round-trips byte-for-byte with SHA-256:

`73b2ae197c314e0b1d0aefb55e0f8c0841536766af63ad9e811bd39e2a273a6f`

## Coverage

The full 1,599-file scan is in
`/tmp/circular_segment_point_coverage.log`. Unsupported slots fall from 777 to
776, complete documents rise from 1,506 to 1,507, complete drawing documents
rise from 1,547 to 1,548, unsupported documents fall from 93 to 92, and partial
views fall from 802 to 801. Embedded-tool coverage remains 4,766/5,356. All
1,599 originals retain byte-exact round trips. No standalone test case was
added.
