# Geometric measurements inside simultaneous mapped iteration

`dev/measurement_mapped44_generate.rs` prepares three native controls and
reconstructs each byte-exact through `add_mapped_point_iteration_with_bindings`.
The raw preparation uses the established two-seed mapped writer, replacing its
carried reference list and the segment binding's source ID. It does not modify
production code or add a decoder exception. The public API reconstruction uses
the same ordinary point, measurement, calculation, and transform records and
passes `verify_roundtrip()`.

All controls have seeds A=(160,200), P=(200,280), fixed O=(80,280),
E=(160,280), and vector sources S=(40,400), T=(120,400). The edited angle
control instead starts A=(160,160). The single simultaneous branch is
`[A,P] -> [A+(T-S),P]`, with fixed depth 3, all levels displayed. P therefore
stays fixed while A advances 80 pixels to the right each generation.

The shared source segment is O–P. Its three carried images are rotated about O
by the angle A–O–E, or dilated about O by the scalar calculation
`length(O–A)/(4 cm)`. The moving segment O–A, its measurement, the scalar
calculation when present, the source segment, the transformed segment, and
A's mapped target are explicit carried prerequisites. They retain their ordinary
DAG references; the measured numeric value must be reevaluated after each state
substitution rather than frozen at its original value.

## Native evidence

Every retained file opened and saved successfully in Sketchpad 5. Both native
queues completed with exit 0, and each saved file compared byte-identically with
its independent raw input and the actual public API reconstruction. The first
queue's initial optional maximize command was unavailable in the headless window
manager; the corrected second queue repeated all three controls successfully.
Retained `.png` files are actual pre-save captures from that final queue. The
complete geometry fits the ordinary native document window; the world-to-capture
offset is (59,51). Captures were inspected directly.

| Fixture under `tests/fixtures/` | Bytes | SHA-256 |
| --- | ---: | --- |
| `measurement_mapped44_angle_native_verified.gsp` | 2746 | `7ab51dff1256295c5f545b0c48141b2ac095528808ef9426041c519071e758e6` |
| `measurement_mapped44_length_native_verified.gsp` | 2924 | `1cf54704c2c28cab224a7c981a6687df192da2d85d8aab237088d3010050de26` |
| `measurement_mapped44_angle_edited_native_verified.gsp` | 2746 | `b5cb75caa1ca3b1a69543c0eb8a81e7bb17851605cc67d8d73d1536f1d30e339` |

For the angle control, all segments start at O=(80,280):

| Generation | Measured angle | Rotated segment endpoint |
| --- | ---: | --- |
| 1 | 26.56505117707799° | (187.3312629199899,226.33436854000504) |
| 2 | 18.43494882292201° | (193.84199576606164,242.05266807797943) |
| 3 | 14.036243467926479° | (196.41710001743982,250.89572499564005) |

The native capture visibly shows these three distinct rays ending at the
predicted coordinates, alongside the initial 45° transformed segment. A pinned
45° measurement would instead overlay every image on the original segment.

The edited control's initial angle is 56.309932474020215°, and its three
angles become 36.86989764584402°, 26.56505117707799°, and
20.556045219583467°. Its endpoints are (176,208),
(187.3312629199899,226.33436854000504), and
(192.35950130828536,237.865187009393), again with start O. The native
capture verifies all three distinct segments after the upstream point changes.

For the length control, the three image endpoints are
(221.99031657123666,280), (280.8046314206921,280), and
(341.81720722672145,280), with fixed start O. These are collinear, so shorter
images overlap the longest in the native rendering. Its visible terminal
endpoint at capture x≈401 agrees with the final recomputed length. Freezing the
initial displayed 2.99 cm measurement would yield x≈169.8 in document
coordinates, far short of the visible final endpoint. This proves length-based
numeric calculation and dilation are reevaluated; individual overlapping middle
endpoints are additionally checked by the public API evaluation, rather than
claimed separately visible in the native screenshot.

Native logs: `/tmp/measurement_mapped44_{angle,length,angle_edited}_native.log`.
Final queue session 35208 terminated with exit 0 after all three independent
`cmp` checks; first queue session 2116 also terminated with exit 0. There are no
pending native probes for these fixtures.

This evidence covers moving-seed angle measurement, moving-seed segment length,
a calculation consuming that length, calculated angle rotation and calculated
ratio dilation of a carried finite segment, two-seed simultaneous substitution,
and an edited upstream source. It does not by itself establish every circle,
arc, polygon, or coordinate-measurement combination. Copying and custom tools
remain outside the requested scope.

## Original corpus CenterArc and PointOnArc mapping

The original paired sample
`../gsp-rs/tests/Samples/个人专栏/贺基旭作品/轴对称案例01（hjx4882）.gsp`
was also opened and saved directly in Native, as an independent original-file
control. Native session 23489 completed with exit 0, and its 7972-byte saved
copy compared exactly with the untouched corpus source. The retained fixture
is `tests/fixtures/mapped_arc44_corpus_native_verified.gsp`, with the actual
pre-save screenshot beside it. Its SHA-256 is
`56d3e366fcbb79cc122be1700bd2c8e319c50ff498448f0351a432b70efbf3c8`.
The paired HTML and raw records were inspected in
`/tmp/graph_cohort44_examples_with_htm.log` (sample section starting at line
414); it records the original construction and rendering context rather than
providing a substitute iteration evaluator.

`dev/mapped_arc44_corpus_probe.rs` prints the current decoded graph and
geometric predictions, while checking the original file's exact encode bytes.
The source arc is ID 25, `CenterArc(3,1,2)`; ID 26 is
`PointOnArc(25,0.42307666666666544)`. Their derived reflection axis is
segment 32. Recovered mapped definition 49 has simultaneous seeds `[2,1,12]`,
images `[35,34,24]`, and carried prerequisites
`[3,24,25,26,29,30,31,32,34,35]`. Its live depth source 33 evaluates to
13.146509290391862, giving 13 images in segment binding 50 (`[32,49]`).
This is an actual recovered mixed point/scalar mapping through a center arc,
point on arc, multiple transforms, and reflection.

The retained screenshot displays the blue reflection axes around the red
figures. Its original stored view has document-to-capture offset (59,68).
Three widely separated generations match the current evaluator:

| Generation | Document segment endpoints | Nearest visible Native blue endpoint pixels |
| --- | --- | --- |
| 1 | (339.68391742441963,293.9585640300664) – (403.9340907808146,221.88403304656654) | (400,361) – (463,290) |
| 6 | (276.1023245192613,460.0610512926663) – (275.80385598712667,556.615355750591) | (335,528) – (335,623) |
| 13 | (299.4435918380844,272.67981592865175) – (322.84140137237904,179.0028961361267) | (358,340) – (382,247) |

Each listed pixel is within 1.7 pixels of the predicted translated endpoint.
This was checked against the RGB values in the actual screenshot after visual
inspection. Some other axis endpoints overlap the original action buttons, so
they are not claimed individually visible. The original advanced root 36 and
its red-figure bindings remain unsupported; the native screenshot's red figures
therefore provide context, not evidence that those separate roots have been
implemented. Root 49 and blue binding 50 are the verified recovered graph.

Native log: `/tmp/mapped_arc44_corpus_native.log`; evaluator coordinates:
`/tmp/mapped_arc44_corpus_evaluation.log`. This original-file control confirms
that the same Wine prefix and native reader successfully handle the corpus
sample while unrelated newly authored path controls undergo their own checks.
