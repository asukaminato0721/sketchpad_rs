# Boundary points on transformed polygon hosts

This code 15 probe is independent of numeric polygon path points (codes 95 and
123). It reuses the original native polygon-boundary object's header, point
label records, and exact 20-byte constraint, changing only its host reference,
edge index, and local fraction. The host is a native-supported
`VectorTranslatedPolygon(5,1,4)` built through the document API from rectangle
`(100,100),(220,100),(220,160),(100,160)`.

The translated polygon has vertices `(100,160),(220,160),(220,220),(100,220)`.
Constraint subtype 53 still encodes its four vertices. Native edge 2 and local
fraction 0.25 visibly place the point at `(190,220)` on C to D, proving local
edge interpolation on the transformed host. The group retains trailer bytes
`ff ff 00 00`. The original controlled fixture opened and saved byte-identically
in Sketchpad (2388 bytes), and is retained as
`transformed_polygon_boundary_vector_native_verified.gsp`.

Decoder and validator use the safe known polygon source-chain count rather than
requiring a primitive polygon. The count requires earlier same-page known
sources and preserves subtype and edge bounds. Reference guards require a
live, earlier, same-page host; the existing finite local-fraction range 0–1 and
trailer checks remain unchanged. Runtime geometry, nearest-edge projection,
and normalized perimeter animation already evaluate live transformed vertices.
The Point tool and Point on Object command now accept known transformed polygon
hosts through that same geometry. Other point host families retain their guards.

Single- and multi-page writers derive the constraint subtype from the known
polygon source chain. Encoding has a separate count helper that permits deleted
objects only while emitting groups before deletion compaction; runtime,
decoder, and validator count checks still require live objects. The helper keeps
all identity, page, strictly decreasing dependency, known-kind, and subtype
checks. A regression deletes a newly authored primitive→vector→boundary chain
and verifies only the three surviving free points remain after save/reopen.

`tests/transformed_polygon_boundary_replay.txt` moves a host vertex, drags the
point onto another edge, copies the point and its dependencies, and creates a
fresh transformed-host point and linked perimeter value through the real API.
The live regression moves C to `(240,180)`: the translated C becomes `(240,240)`,
so edge 2 at 0.25 moves the point to `(205,235)`. Dragging toward `(105,190)`
projects onto edge 3 at 0.5, yielding `(100,190)`. Copying gives `(120,210)` with
the standard paste offset and a remapped transformed host. The editor command
also constructs the midpoint of edge 0, and save/reopen retains that constraint.

The actual host-vertex edit also opened and saved byte-identically (2388 bytes).
Its successful visible retry capture
`/tmp/transformed_polygon_boundary_host_vertex_retry_before_save.png` shows
`(205,235)`, not just file acceptance. The dragged output opened and saved
byte-identically (2388 bytes); its capture shows `(100,190)` on edge 3 at 0.5.
The fresh API-generated point, marker, and linked Value of Point opened and saved
byte-identically (2458 bytes). These fixtures are retained as
`transformed_polygon_boundary_{host_edited,dragged,generated}_native_verified.gsp`.

The first copied output exposed a real writer defect: a new code-15 polygon
point put its `0x07d5` label and `0x07d8` font before references and constraint,
while the actual native group puts them after the constraint. The file produced
a native format-error alert. A controlled candidate moved only those two record
blocks; it then opened and saved byte-identically (2932 bytes), with both
original and copied points visibly correct. This isolates the cause without
altering the header or geometry. The single- and multi-page PointOnPolygon
writers now emit labels after the constraint; other object families are
unchanged. The independently native-verified candidate is retained as
`transformed_polygon_boundary_copy_label_order_native_verified.gsp`, and the
regression compares the actual copied writer output to all its bytes. Failed
captures remain under `/tmp/transformed_polygon_boundary_copied_port_open_failed.png`;
a separate early drag failure left the file-open dialog visible and succeeded
on retry, so that failed attempt was not counted as native proof.

The corrected actual document writer was then replayed with the C vertex edit,
point drag, and copy. Its copied output independently opened and saved
byte-identically (2932 bytes), retaining both original and remapped constraints.
The visible capture `/tmp/transformed_polygon_boundary_copied_port_before_save.png`
shows the original at `(100,190)` and the copy at `(120,210)` on their respective
transformed hosts. This actual output is retained as
`transformed_polygon_boundary_copied_native_verified.gsp`; the manually ordered
candidate remains separate so the isolation proof and API proof are distinguishable.
