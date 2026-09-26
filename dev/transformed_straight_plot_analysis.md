# Value and literal points on fixed transformed rays and lines

Native code 95 and 123 were tested independently on code 27 fixed rotated and
code 30 fixed scaled ray/line hosts. The controls preserve the existing 64-byte
value-point and 80-byte literal-point schemas and change only native references
(and code 123's literal). Each contains both record families at 0.25, -0.25, 1.25
and ordinary marker segments from a defined free point to each plotted point.

For rotation, the primitive start/end (170,190), (210,190) are rotated 90 degrees
about (130,150), giving host controls (170,110), (170,70). Both native record
families put 0.25 at (170,100) and 1.25 at (170,60). The line's negative -0.25
point is (170,120). On the ray, the negative point and its dependent marker
segment are absent; there is no point at the start or negative affine position.
Native retains the finite negative value and saves the whole file unchanged.

For scaling, factor 2 about (130,150) gives controls (210,230), (290,230). Both
record families put 0.25 at (230,230), 1.25 at (310,230), and line -0.25 at
(190,230). The ray's negative point and marker disappear while positive markers
remain. The native ray does not clamp negative plotted values to the origin.

All four scripts exited successfully before byte comparison. Each rotated
control preserved 4280 bytes; each scaled control preserved 4256 bytes. The
retained fixtures are `rotated_{ray,line}_path_values_native_verified.gsp` and
`scaled_{ray,line}_path_values_native_verified.gsp` in `tests/fixtures`. Captures
are `/tmp/{rotated,scaled}_{ray,line}_path_values_native_before_save.png`.

The host whitelist is extended only to the four proved kinds: RotatedRay,
RotatedLine, ScaledRay, ScaledLine. Evaluation uses the live transformed control
points and the affine formula `start + value*(end-start)`. Rays require a
nonnegative value for a defined point; finite negative values remain valid
undefined 95/123 states. The API preserves the negative value instead of
clamping it, and validates the live host before checkpointing. Validation
retains scalar-source, finite-value, earlier/live/same-page host/source checks;
the missing-position exception is restricted to negative values on the proved
transformed rays with defined unequal host control points. Primitive host
policies and other transformed straight families remain unchanged.

The native invisible marker segments independently establish the supported
undefined dependency state. New Segment endpoints must each be live, earlier,
same-page points that are either defined or directly one of the proved negative
95/123 ray points. Existing dependency guards for Line, Ray, Circle, Midpoint,
and other families remain unchanged. This permits faithful copies of native
marker segments without a blanket missing-position exception.

`tests/transformed_straight_plot_replay.txt` authors all four host families and
both record families through the public API, then moves a source endpoint and
copies the complete dependency graph. Focused regressions cover native import,
positions and definedness, value edits, undo/redo, save/reopen, paste remapping,
dependent deletion, and atomic rejection of nonfinite values/invalid hosts.
The existing guard against deleting an earlier object before a surviving plot
is retained; deletion coverage removes the copied host and its dependent plots.

The actual authored generated and source-edited files each opened and saved
byte-identically at 6386 bytes, and the copied 120-object graph at 11004 bytes.
The retained `transformed_straight_plot_{generated,edited,copied}_native_verified.gsp`
fixtures are compared byte for byte against public API construction, source
movement, and copying in a regression. Native screenshots show the source edit
changing both transformed paths and their marker segments, and the copied
geometry offset by (20,20). Undefined ray plots and their markers stay absent.

A separate actual API diagnostic avoids coincident positive markers between
the two record families. Code 95 uses 0.25, -0.25, 1.25; code 123 uses 0.5, -0.5,
2.5. The 0.25 and 0.5 endpoints are separately visible for every host. Negative
line markers are separately visible while negative ray markers remain absent.
The rotated 2.5 point is also visibly at (170,10); the scaled 2.5 marker continues
toward its endpoint beyond the right viewport edge. This diagnostic opened and
saved byte-identically at 6386 bytes, retained as
`transformed_straight_plot_distinct_native_verified.gsp` and reproduced by
`tests/transformed_straight_plot_distinct_replay.txt`.
