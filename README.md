# Sketchpad RS

Experimental Linux editor for Sketchpad 5 `.gsp` documents. It uses winit and egui for the window and controls, and wgpu with a WGSL shader for sketch geometry.

## CJK text

Chinese (simplified and traditional), Japanese, and Korean text use system font
fallbacks in the interface, text fields, and canvas, including captions with an
explicit font family. Font lookup uses `fc-match` from fontconfig at startup;
install CJK fonts such as Noto Sans CJK and restart the editor after installing
fonts. The Nix development shell supplies fontconfig but uses your installed
system fonts. Without fontconfig or suitable fonts, the editor still starts,
but unsupported characters may appear as missing-glyph boxes.

Text fields use the desktop input method through egui/winit. Configure and start
your desktop IME (for example, Fcitx5 or IBus) to compose CJK text. Captions and
point labels preserve Unicode when editing, copying, and saving `.gsp` files.

## Run

```sh
nix develop -c cargo run --bin sketchpad_rs -- path/to/sketch.gsp
```

For a quick interactive demonstration, generate the centered sample and open it:

```sh
nix develop -c cargo run --bin gsp_probe -- demo /tmp/sketchpad-demo.gsp
nix develop -c cargo run --bin sketchpad_rs -- /tmp/sketchpad-demo.gsp
```

Drag points A or B to see the segment, midpoint, circle, line, ray, parallel,
and perpendicular constructions update together. The development shell also
includes `xvfb-run` for unattended desktop smoke runs.

The path is optional. The Nix shell supplies the desktop libraries and fontconfig needed at runtime. Use the File menu to create, open, save, close, and quit; Ctrl+S saves the current file and Ctrl+Shift+S opens Save As. New, Open, Close, and Quit prompt about unsaved changes. File > Document Options adds, copies, renames, removes, and reorders pages in supported sketches; Undo and Redo handle these changes. Page removal asks for confirmation. The File menu imports a single PNG picture, exports the visible page as SVG or PDF, and sends it to the Linux default printer. Page Setup selects A4 or Letter paper, orientation, margins, and print scaling; Print Preview shows the same page image used by PDF export and printing. PDF output and preview require `rsvg-convert`; printing also requires `lp`. The toolbar creates points, segments, circles, and literal text. Clicking a supported segment, ray, line, circle, or arc with the Point tool creates a point constrained to that object, including constructed straight paths and transformed segments/circles. Value of Point measurements on these paths can feed live calculations and iteration depth. Select the required objects for the Construct commands. The Measure menu adds live measurements, including length, two-point and point-to-straight-object distance, area, perimeter, point value on a segment, ray, line, circle, or arc, signed point ratio, and slope of straight objects (including constructed lines and transformed segments) in a Cartesian coordinate system. The Number menu supports parameters, source-free calculations using numbers, decimals, arithmetic, powers, functions, π, and e, live multiplication of a selected length measurement or length calculation by 0–99 or division by 1–99, arithmetic on two selected lengths using a + b, a - b, a * b, or a / b in selection order, and live expressions of a selected numeric parameter or a prior parameter calculation, plus parameter tables with row controls. Number > Attach Selected Value to Point creates a live numeric label for a selected point and parameter or calculation. To plot a point in a supported one-page sketch, define a coordinate system, then choose Cartesian or polar Plot Points from Graph and enter coordinates. The dialog can plot successive points without selecting a table. Display changes supported point styles, line widths and patterns, colors, and text font families; it shows or hides existing labels on selected points, straight objects, and circles, traces selected points and segments while dragging, and clears the painted trails with Erase Traces. Edit > Cut, Copy, and Paste use an internal sketch clipboard for supported objects, and Copy and Paste also work for verified constructions across supported multi-page pages; Edit > Clear or Delete removes selected objects and their dependents in supported single-page files and supported multi-page construction and measurement groups. The Transform menu can translate or rotate a point, reflect a point or circle across a selected line, dilate a point or circle around a selected center, and iterate a selected free point with its directly translated, rotated, dilated, or reflected image to a chosen depth from 1 to 128, optionally carrying the segment between them. Selecting the connecting ray and a scalar parameter as well makes the iteration depth follow the parameter. To iterate a numeric value along with the point, select the seed point, transformed image, parameter, direct parameter calculation, and numeric label attached to the image, then choose Iterate. Selecting a second scalar parameter makes its value control the numeric iteration depth. To branch a triangular interior, select its free seed, two directly translated image vertices, the interior, and optionally its sides and a scalar depth parameter. Drag a point to update its dependents or its position on a segment, ray, line, circle, or arc. Scroll over the canvas to zoom. Multi-page files have page controls in the left panel. Saving preserves the selected page for reopening, including navigation through Page Link buttons. The Point, Segment, and Circle tools, plus the Construct commands for lines, rays, midpoints, parallel and perpendicular lines, straight and circle intersections, angle bisectors, three-point arcs, arcs on circles, polygon interiors, and circles with a segment radius, work on supported multi-page pages. Points can be constrained to newly created paths. Supported multi-page pages can also add length, distance, perimeter, circle, polygon, arc, angle, radius, and point-ratio measurements. Existing points can be moved, and changes to existing object styles and trace flags can be saved. Literal captions can also be created, edited, moved, formatted, copied between supported pages, and cleared; mixed character formatting and font families survive saving. See [multi-page text checks](dev/multi_page_text_analysis.md).

Graph also offers Square, Polar, Rectangular, and trigonometric X-axis grid forms, Show Grid, Dotted Grid, and Snap Points for a defined coordinate system. Properties can independently label either axis with decimal numbers or fractions and multiples of π, including circle- and distance-defined frames. A one-distance frame can switch to Rectangular and drag its vertical unit independently; a two-distance frame can switch between Rectangular and Polar. Graph > Plot Value on Axis/Path accepts a number in its dialog, or plots a live value when t₁ is selected. Select a segment, ray, line, circle, or axis to use it; otherwise the active X axis is used. Segment values run from 0 to 1, ray values start at 0, and circle values count counterclockwise turns from the rightmost point.
Selecting a point before Graph > Define Coordinate System uses it as the origin, including when the sketch already has a frame. Selecting a circle defines a square unit frame centered on that circle, with the circle's radius as one unit. Select one segment or distance measurement to define a square unit scale, or two in selection order to define separate horizontal and vertical scales; a selected point can set the origin. Select an origin, unit point, axis, or grid intersection and choose Graph > Mark Coordinate System to make that frame active for subsequent plots and coordinate measurements. Snap Points leaves the visible grid on screen.
When a sketch has multiple frames, the editor and SVG export draw each visible frame using its own Square, Rectangular, or Polar grid form and dotted setting.
The Number menu can create t₁ and t₂ parameters or multiple named ASCII parameters in a supported single-page sketch, and edit finite values with up to 32 native numeric tokens. Parameter captions support 0–5 decimal places and use scientific notation at magnitudes greater than one million; calculations retain the full value. Decimal edits increase display precision up to five places. Supported values can be tabulated into tables with snapshot row controls; tables remain limited to −99.99 through 99.99 with two decimal places. The default one-column t₁ table caches values from −99.99 to 99.99; fractional, negative, two-digit, and later-source snapshot files cross-opened in Sketchpad 5. The supported two-column table uses t₁ and t₂ values from −99.99 to 99.99 with up to 256 rows; generated five-row and mixed-width files opened and saved byte-identically in Sketchpad 5. Graph > Plot Table Data plots its Cartesian or polar rows, including distinct and fractional values.
For a selected source-free calculation, function, or numeric parameter, Edit >
Edit Definition opens the corresponding value or expression editor. Edit >
Properties shows the selected object's saved label, hidden state, and whether
the Arrow tool can select it, with links to inspect its direct parents and children.
Point, straight-object, circle,
base-arc, and transformed straight/circle/arc Properties can rename and show or
hide their labels. The same controls cover live colorized straight, circle, and
arc outputs; polygon-interior Properties can change opacity. Hide/Show button
Properties can edit its caption, fixed or alternating action, fade behavior,
and post-show target selection. Animation button Properties can edit its
caption and each target's direction, Once Only state, speed, and parameter
timing fields. Movement button Properties can edit its caption, speed, and
follow-moving versus initial-destination policy. Scroll button Properties can
edit its caption and Upper Left versus Center placement. Presentation button
Properties can edit its caption, execution mode, pre-actions, stop policy, and
elapsed or pause time. Link button Properties can edit its caption, URL, target
page, and optional destination action, including URL/Page conversion in
multi-page documents. Sound button Properties can edit its caption while
retaining its fixed mono or stereo function sources. Iteration-image Properties
can change a fixed iteration count and choose all images or only the final
image; parameter-driven counts remain live and are edited through their source.
Axis Properties can independently choose decimal tick labels or fractions and
multiples of π for x or y axes. Sampled transformed-picture Properties exposes
Native's sample count and None, Some, Most, or All allowed-distortion choices.
Function and parametric Plot Properties edit a 20–250000 sample count, numeric
domain, continuous or discrete drawing, and arrow/endpoint visibility. Discrete
plots render as individual samples; continuous plots retain endpoint arrows.
Edit > Split Point detaches a supported constructed point from its parents in a
safe single-page sketch while keeping its dependent constructions attached.
Select an independent point first and a target point second to use Edit > Merge
Points, which transfers the source point's children to the target.
Edit > Preferences provides selection magnetism, optional double-click
deselection, and canvas background and selection colors for the current editor
session. It can also open the Text Palette automatically when a caption is
created.
To define a point transform, select a pre-image point and then a dependent image
point and choose Transform > Define Custom Transform. The named command appears
at the bottom of the Transform menu and maps one selected point through the same
live construction relationship. Transform > Edit Custom Transforms renames,
reorders, or removes definitions. Applied points use the native mapped-element
format and survive save/reopen; the definition list itself currently lasts only
for the open document session.
Select an independent axis-aligned PNG picture and choose Number > Define
Function from Picture to create a data-defined function from the picture's
nontransparent top edge. A default coordinate system is added when needed; the
function can be evaluated by supported calculations and survives native save.
Custom Tools > Choose Tool Folder loads embedded tools from top-level `.gsp`
files in a folder for the current application session. Folder tools use the same
selected-input and click-input workflows as document tools, while Forget Tool
Folder removes only the catalog and leaves completed constructions intact.
All seven corpus picture-bearing tool prototypes can materialize their bounded
point-attached PNG outputs, including a seven-resource construction; arbitrary
imported multi-picture resource tables remain read-only.
Display > Animate Objects runs a selected parameter with its stored continuous/discrete mode, rate, and domain, or animates a supported point, including transformed points, points on polygon and sector boundaries, and supported point curves. Edit Selected Parameter and the common Properties panel expose the current value, saved precision, name/value display mode, inline edit box, those animation settings, and Native's keyboard adjustment; `+` and `−` apply that adjustment without restricting manual values to the animation domain. Parameters with an inline edit box retain their border and open the editor on canvas double-click; disabling it removes the border and direct canvas editing. A transformed point uses temporary motion state, so its construction is unchanged and Stop returns it to its constructed position. Selecting a construction such as a segment or circle animates its source points, even when those points are hidden. Edit > Action Buttons > Animation creates a clickable motion button for selected parameters and points, or for a selected segment, line, ray, circle, or polygon with up to seven vertices through its source points. Animation buttons for supported points and segment sources can also be created on existing multi-page sketches. Parameter buttons can use continuous or discrete changes with an editable rate, step, and domain. The Motion Controller pauses, resumes, changes speed, and stops Display animation.
Edit > Action Buttons > Sound creates a mono button from one selected function
or a stereo button from two selected functions. Clicking the button toggles
playback; the function variable is time in seconds, and the first and second
functions supply the left and right channels.
Select a caption and highlight characters in the sidebar text field to apply Bold, Italic, Underline, Larger, Smaller, or a font family to that range. These mixed-format captions render in the editor and SVG, and survive word edits, move, and copy/paste. Text insertions inherit the neighboring style, and replaced text inherits the first replaced character's style.
Display > Show Text Palette opens floating font-family, emphasis, and size controls
for selected text, point labels, and numeric captions. Hiding or closing the
palette does not modify the sketch.
Imported captions also support native angle, triangle, similarity, implication, and multiplication symbols, plus superscripts. Superscript formatting survives text edits and appears in both the editor and SVG output. Chained geometric transforms accept constructed sources and centers, including transformed polygons and sectors.
Imported mapped iterations can carry nested iteration descriptors, point-anchored pictures, and sectors. Picture instances share their source texture and follow their scoped anchors in the canvas, picking, and SVG export; imported point attachments retain their native scale and rotation. Imported picture transforms also compose live translations, rotations, dilations, and reflections through the shared affine evaluator. Fully decoded active pages can be exported even when structural editing is restricted.
Polygon interiors can be reflected, rotated by a fixed or marked angle, dilated by a fixed or marked ratio, translated by fixed or marked polar and rectangular distances or a marked vector, and measured for live area. These results update when their source vertices or defining points move, and the generated files have been opened and resaved in Sketchpad 5.

Imported Point transforms also support Native code-22 fixed-distance polar
translation by a live three-point directed angle. The two corpus roots and a
dependent vector point evaluate through the shared transform model; unchanged
files remain byte-exact, while fresh authoring and clipboard reconstruction
stay guarded. See [the bounded evidence](dev/point_angle_fixed_polar_analysis.md).

Select standard length, distance, perimeter, circumference, radius, area, angle, arc, ratio, x/y coordinate, coordinate distance, slope, or Value of Point measurements and use **Measure → Decimal Places** to choose 0–5 decimal places. Precision is saved separately for each measurement and preserved by undo and copy/paste. Supported single-page calculation captions also accept 0–5 decimal places, including literal formulas, source-dependent formulas, measured-length arithmetic, and two-length arithmetic. Mixed selections change together with undo and copy/paste support. Calculations keep using the full measured values. The same precision appears in desktop captions and SVG exports, including measurements on supported multiple-page sketches.

Selected numeric values also support `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `abs`, `sqrt`, `ln`, `log`, `sgn`, `round`, and `trunc` in calculation expressions, including nested calls. Trigonometric arguments use degrees; inverse functions accept scalar arguments and return degrees usable as marked rotation angles. Constants `pi` (or `π`) and `e` can appear alongside the selected source. With no source selected, the calculator accepts the same functions and constants, including signed numbers and degree literals. Scalar formulas can supply iteration depth; degree formulas can supply marked rotation angles. `log` uses base 10. `abs` preserves the source’s length, area, or angle units. The remaining non-trigonometric functions currently require scalar arguments. Undefined results stop dependent geometry until the source is valid again.

Numeric iterations can use a formula with multiple inputs. Select the free point, its image, the parameter to iterate, the scalar formula, and the formula’s label attached to the image. Other formula inputs stay at their current values during the iteration. Editing an external input refreshes the whole sequence. For example, `k+x` advances by `k` on each step when `x` is the mapped parameter.

Select two to sixteen numeric measurements, parameters, or calculations to combine them in one formula. Use `v1`, `v2`, … in selection order; `a` and `b` also refer to the first two. Examples include `(a+b)/2` for two lengths, `v1*v2` for a length scaled by a parameter, and `sqrt((v1+v2)*v3)` for three scalars. Results retain compatible units and update when any source changes. Scalar results can control iteration depth; degree results can control rotation. Expressions remain limited to 32 native tokens.

To carry a circle through an iteration, select a free center point, its directly translated, rotated, dilated, or reflected image, and the circle centered on the seed through that image. Iterate uses the dialog depth (1–128); also select a scalar parameter or calculation to control depth live. Calculated rotation angles such as `360°/n` and a depth such as `n-1` update the circle orbit together. Repeated circles retain their binding’s stroke style and can be selected or hidden. Dilated mappings use a fixed scale, a marked three-point ratio, or a marked scalar parameter/measurement/calculation. Centers and ratio inputs can be supported constructed points. Repeated circles scale their radii with each step; negative factors alternate sides of the center. The same point mappings support point, segment, ray, circle, and combined point/parameter iterations. Reflected mappings alternate across the current mirror. Vector mappings update with their endpoints; using the seed as the vector start fixes later points at the endpoint, while using it as the endpoint doubles the displacement on each step. Marked polar and rectangular distances, either mixed rectangular translation, and three-point marked angles also support iteration. Length calculations can supply the marked distances. Every direct seed reference is substituted at each step, including marked angle and ratio inputs; external construction inputs stay fixed within each sequence.

Removing a page leaves Page Links to that page visible with an unavailable destination, as in Sketchpad 5. The confirmation names affected links.

## Current scope

Construct > Locus accepts a selected point constrained to a segment, circle,
arc, polygon/sector boundary, or supported point curve (including function
plots and intrinsic sampled mappings), and a dependent point, segment, ray, line, circle, arc, or polygon
interior, including an interior translated by a marked vector. Point loci
follow supported dependency graphs; other loci draw sampled object families.
The line targets include lines constructed
parallel or perpendicular to another line; angle bisectors produce ray
families.

Display > Animate Objects moves selected free or transformed points, or points constrained to
segments, rays, lines, circles, arcs, polygon/sector boundaries, or supported
point curves and updates their dependent constructions. Free
points wander within the visible canvas; segment and arc points reverse at
their endpoints, ray and line points reverse at the visible path ends, circle,
polygon, and sector points loop, and generic point curves reverse. The Display menu and Motion Controller
can change speed, pause, resume, and stop the motion. Motion keeps the point's
current position when stopped and can be undone as one edit. Transformed-point
motion is temporary: Stop restores the construction-defined position, and Save
does not serialize the runtime override.

Edit > Action Buttons > Hide/Show creates one button for all selected supported
objects, including on an existing supported multi-page page. Clicking it hides
or shows those objects together.

Edit > Action Buttons > Animation creates one button for the selected animatable
points, including on a supported existing multi-page page. The dialog sets
direction, speed, and Once Only for each point. Clicking the button starts or
stops their motion together.

Edit > Action Buttons > Sound creates one button for one or two selected
functions, including on a supported existing multi-page page. One function is
played as mono; two functions supply left and right stereo channels. Clicking
the button toggles playback, and Escape stops all active Sound buttons.
Properties edits its caption without changing those channel sources. See the
[Sound-button evidence](dev/sound_buttons_analysis.md).

Edit > Action Buttons > Movement takes a selected free point followed by a
destination point, including on a supported existing multi-page page. Its
dialog sets Slow, Medium, Fast, or Instant speed and whether to follow a moving
destination. Clicking the button moves the source point; clicking again stops
an active movement.

Edit > Action Buttons > Scroll takes one selected point, including on a
supported existing multi-page page. Its dialog places the point at the canvas
corner or center when the button is clicked.

Edit > Action Buttons > Presentation takes selected action buttons in selection
order. Its dialog chooses simultaneous or sequential activation, optional
deselection/animation-stop/trace-erasure before starting, and the native stop
or pause timing policy. Sequential presentations wait for a running action to
finish and any configured pause before advancing.

Edit > Action Buttons > Link creates a URL button or, on any page of a supported
multi-page document, a Page Link. The editor opens HTTP(S) and file links with
the system handler, opens `sketchdoc://` links in the current document's folder,
and follows Page Links within the document. A Page Link can also activate a
Hide/Show button on its destination page. The resulting Hide/Show state can be
saved in a supported multi-page document. The destination button and Page Link
can be created together before the first save.

The writer supports the constructions and measurements listed in [COMMAND_PARITY.md](COMMAND_PARITY.md), including polygons, signed point ratio, slope, Cartesian arithmetic and sine/cosine/tangent function plots, rectangular and polar parametric plots from two functions, editable plot domains and sample counts, a single t₁ point plotted in Cartesian or polar coordinates, live polygon-cropped pictures, Hide/Show, Animation for up to seven verified source points including mixed free, segment, and circle points, one- or two-function Sound, one-pair point Movement, one-point Scroll, Presentation, URL Link, and Page Link action buttons on supported multi-page documents, selected-object styling, and bounded parameter/table workflows. It can update coordinates, supported point constraints, visibility, and styles in place in existing documents, and remove selected objects with their dependents in supported single-page documents. Copy includes a selected object's construction parents; Paste duplicates them with a 20-unit offset. Transform > Mark Center and Mark Mirror let a later rotation, dilation, or reflection use one selected source object. It preserves unsupported source records byte for byte. Structural edits to a document with unsupported content are refused when saving. Saves write through a temporary file in the destination directory and rename it into place.

Other Sketchpad features are incomplete. In particular, many transformations, graphing commands, multi-picture and attached-picture workflows, advanced table workflows, custom tools, action button variants, other import formats, and full multi-page editing are still pending. See [COMMAND_PARITY.md](COMMAND_PARITY.md) for the command inventory. This build is not ready as a Sketchpad 5 replacement.

## Workspace layers

The workspace has four implementation crates and the original application facade:

- `sketchpad-format`: lossless native record framing and page boundaries, with no UI or model dependency.
- `sketchpad-ir`: generic object nodes, semantic geometry/transform expressions, and the derived dependency index. It has no file-format or UI dependency.
- `sketchpad-engine`: semantic decoding, document evaluation and scoped substitutions, editing/history, and byte-preserving saving. It depends on format and IR, and has no production GPU/window dependency.
- `sketchpad-editor`: desktop commands, dialogs, animation, picking and rendering over the engine.
- The root `sketchpad_rs` package preserves the previous public module paths and starts the desktop application.

The IR object graph is authoritative. The dependency index derives forward and reverse edges and a topological order; it does not maintain a second editable graph. Element transforms retain live numeric/geometry references until evaluation. Native offsets and retained source bytes remain engine provenance.

Sources live under `crates/<crate>/src/`. Binary fixtures remain in `tests/fixtures/`. Existing engine tests compile the shared renderer source as a test-only verification bridge; this does not add a production editor dependency to the engine.

A headless coverage/inspection build omits the desktop dependencies:

```sh
cargo build --no-default-features --bin gsp_probe
cargo run --no-default-features --bin gsp_probe -- coverage ../gsp-rs/tests
```

SVG export uses the default desktop feature. The GUI entry point remains `cargo run --bin sketchpad_rs`.

Live referenced captions can show values, selected source names, definitions and expressions, including text/function sources and captions attached to points. Generated measurement names retain their assignment through hiding, undo and save/reopen. Rich fractions currently use grouped division in plain presentation. See [scope and evidence](dev/referenced_shared64_implementation.md).

Mapped iterations also support filled Circle interiors through the shared scoped evaluator, with live or frozen Circle geometry, final-only output, picking, independent binding styles and undo. See [implementation and verification](dev/circle_interior_iteration65_implementation.md).

Anchored Value-reference text can iterate with live or frozen numeric/text/function sources. Nested single-point terminals inherit outer substitutions when their descriptor is carried, and retain a frozen entry environment when it is omitted. Complete active capture, local seed shadowing and undefined terminals use shared validation and evaluation. See [scope implementation](dev/shared_scopes66_implementation.md).

Carried straight-output Locus collections now replay their local sweep inside each outer iteration scope. Editing, styled rendering, picking, save/reopen and selection-based authoring use the same scoped result. See [collection implementation](dev/scoped_collection67_implementation.md).

Carried sampled point-curve collections use that scoped path as well. A source curve is evaluated within each outer iteration generation, with unrelated parent substitutions preserved; editor picking and rendering consume its styled segments. An original circular-chain document and a public edit match Native saves byte-for-byte, and visibility controls isolate the curve binding's painted geometry. See [sampled collection implementation](dev/sampled_collection68_implementation.md). The imported example remains a partial view for separate structural reasons, so its SVG export is guarded.

Function plots can also participate as upstream recipes in mapped point/shape constructions. Their function and coordinate frame are resolved separately against the carried outer scope; an omitted plot wrapper does not itself freeze a carried frame. An actual triangle-iteration sketch and public edit match Native saves and independently checked generated points and segments. See [mapped plot implementation](dev/mapped_function_plot69_implementation.md).

Point-output Locus collections can be carried by an outer mapping. Each generation samples its own trace, while the direct source Locus remains a separate visible object. Editing a parameter can therefore reveal a source trace without adding a binding generation. Native original/edit/hide controls and public writer states are recorded in [point-Locus collection implementation](dev/scoped_point_locus70_implementation.md).

Referenced text now evaluates Native π suffix and numeric subscript operators from live values while preserving their original rich records and cached text. The imported originals and public editor controls cover value changes, display precision, SVG, picking and save/reopen; unrelated partial-view guards remain in place. See [rich-token implementation](dev/referenced_token71_implementation.md).

Referenced functions also parse grouped unary arguments with a following operator, such as `trunc((Y−1)/x)` in an original calendar. The decoded function and downstream calculation evaluate from live sources without changing the original file bytes; a clean authored control covers parent edits, undo and reopen. Native save and value checks, corpus gains, and the remaining nonfinite-value boundary are recorded in [the implementation notes](dev/referenced_function72_implementation.md).

Standalone code26 custom-transform points with a Free source and live Degrees/Length operands now reuse the generic polar transform evaluator. Native controls from two originals verify source movement, angle and distance changes, including zero and negative distance; trace-linked and Segment forms remain separate. See [the bounded implementation](dev/code26_standalone73_implementation.md).

The same standalone operation also follows a Vector-transformed Point source. A separate Native original isolates the output marker and verifies its movement; [the evidence](dev/code26_transformed74_implementation.md) keeps this source type distinct from unproved trace-linked and other transformed variants.

Referenced functions also accept Native expression wrappers with a bounded high-word count field. The parser validates the low token count and full expression, while edits retain the original high word. Two Native originals exercise high1 and high2 live sources; [the implementation notes](dev/referenced_function75_implementation.md) record the byte-preservation and partial-document limits.

Standalone code26 points also follow a FixedPolar-transformed Point source. A Native original and four byte-exact save states isolate its marker and live upstream motion; [the bounded extension](dev/code26_fixedpolar76_implementation.md) retains the earlier source and record guards.

The same operation now follows a MarkedPolar-transformed Point source. Native controls also exposed a separate segment PointValue boundary rule; [the evidence and regression](dev/code26_markedpolar77_implementation.md) distinguish constrained points from free-point projection.

Scalar x/y coordinate measurements now preserve a Native empty marker after their style record. A visible x-value changes live in a byte-exact Native original, while a paired x/y original verifies the marker's retention under Native normalization; [the coordinate-measurement evidence](dev/coordinate_measure_analysis.md) records their downstream calculations and mappings.

An intrinsic sector boundary can now be fixed-rotated as a live analytic element. Native 180°/90° controls and an upstream move verify the [strict transform case](dev/sector_transform79_implementation.md), while the parser still preserves unproved sector operations as unsupported records.

The same element now supports a proved [Segment reflection and chained rotation](dev/sector_transform80_implementation.md), keeping both source references live through the matrix sequence.

[Repeated sector rotations](dev/sector_transform81_implementation.md) replay their live matrices in construction order; the strict Native-backed decoder restores long chains without adding shape-specific operation variants.

[Imported polygon-walk points](dev/polygon_walk82_implementation.md) now follow the source's ordered stored edges, including repeated vertex references, with strict subtype and record-layout guards.

[Circular-segment boundaries](dev/circular_segment83_implementation.md) now evaluate a live Arc and its endpoint chord as one filled region, retaining strict native record admission and dependency order.

[Directed three-point angle values](dev/angle_values84_implementation.md) now retain separate positive and reversed 360° conventions while sharing one live point-based calculation.
[Named angle aliases](dev/angle_alias85_implementation.md) bind a displayed Degrees value to either verified angle source; default and explicit names follow the live parent without duplicating its angle convention.
[Colorized filled disks](dev/colorized_disk86_implementation.md) reuse the shared live color evaluator. Disk and sector fills support spectrum, grayscale, RGB, and HSV bindings, including live numeric operands and copied constructions.
[Three-point angle markers](dev/angle_marker_presentation.md) now draw native-sized sectors, multiple arcs, right-angle squares, and optional arrowheads. Their stored label positions, opacity, and arrow flags survive copying and saving; generic numeric aliases retain the page's degree or radian unit.
[Mapped colorized disks](dev/colorized_iteration88_implementation.md) reuse the scoped iteration evaluator to draw an independently colored, analytic filled disk at each generation.
[Colorized polygon regions](dev/colorized_polygon89_implementation.md) now use the same live geometry/color evaluation, including a distinct fill color for each mapped generation.
[Imported URL buttons](dev/url_suffix90_implementation.md) now preserve opaque native bytes after the URL terminator in the observed action records, restoring their linked text without discarding those bytes on ordinary edits.
[Multi-resource PNG documents](dev/image_objects91_implementation.md) now show independently indexed pictures using the existing image renderer; these imported resource tables remain read-only.
[Native colored text runs](dev/rich_style92_implementation.md) now retain ordered R/G/L colors on individual spans in canvas and SVG output.
[Referenced functions with π suffixes](dev/referenced_function93_implementation.md) now decode through the shared expression evaluator, restoring live dependencies in the imported music examples.
[Point-anchored PNG pictures](dev/anchored_image94_implementation.md) now follow free or constructed points using the shared image renderer and SVG/picking transform.
[Two- and three-point attached PNG pictures](dev/image_analysis.md) now follow their live rectangle or parallelogram corners through the same renderer, SVG, picking, and iteration paths; imported resource tables remain read-only.
[Nested polar point transforms](dev/point26_dual95_implementation.md) now compose through the existing object DAG when a code26 point uses an already decoded polar point as its source.

Imported polar translations also accept constructed and path-constrained source points. Rolling-circle functions accept native numeric suffixes, and captions retain their auxiliary object names. Internal page buttons preserve native label padding and optional metadata. Existing geometry, numeric values, and visibility can be saved in documents containing imported pictures through the preserving writer.

## Verification

Sampled mappings now follow polygon boundaries and constructed rays/lines, and mapped iterations carry numeric quantities with matching units. Curve labels retain their fonts through copy/reopen. See [implementation and native checks](dev/sampled_paths_and_quantity_iterations.md).

Trigonometric calculations, rotation angles, and dilation ratios now consume numeric magnitudes regardless of source units, matching native Sketchpad. Angle helper objects and their captions can also be copied and saved. See [quantity operand verification](dev/numeric_quantity_operands.md).

Parameters accept leading decimals, π/e constants, and scalar or typed scientific literals with integer exponents. Their dependent calculations remain live through edits and copying; see [native literal checks](dev/parameter_literal_forms.md).

Run `cargo test --workspace` to include every implementation crate. Tests cover record parsing and byte preservation, dependency updates, geometry, undo/redo, and save/reopen. All 1,599 `.gsp` files in the sibling project and the bundled 523-object Example Tools document pass unedited byte-preservation verification; many remain partial views and are guarded against unsafe edits. Generated constructions, measurements, text, action buttons, styles, bounded number/table files, and arithmetic and sine/cosine/tangent plots have been opened and saved in the supplied Windows Sketchpad 5 through Wine. Native KDE screenshots confirm geometry and style rendering; full command and interaction verification remains open.

`bash dev/native_desktop_smoke.sh path/to/sketch.gsp /tmp/sketchpad_capture` launches the built binary and captures the current desktop. When Nix is available, the script enters the development shell automatically to load its Wayland libraries.

`gsp_probe` provides headless `inspect`, `diff`, `verify`, `coverage`, `demo`, `replay`, and `svg` commands for comparing original and generated files and exercising document commands. Integrated Cartesian and polar t₁ Plot Points files were opened and saved byte-identically in Sketchpad 5 under Wine. The Ghidra scripts and current binary analysis are in [dev/ANALYSIS.md](dev/ANALYSIS.md).

### Regular polygon iteration

Create a numeric parameter `n` (for example, 5), then select it and use Number → Calculate to create `n-1` and `360°/n`. Select the degree calculation and use Transform → Mark Angle. Select a seed point followed by a center point and rotate using the marked angle. Connect the seed to its rotated image with a segment. Select the seed, image, segment, and `n-1` calculation, then use Iterate. Editing `n` changes the angle and iteration depth together; live depth is truncated to a whole number from 1 to 128.

A measured angle or arc angle can be selected directly with Transform → Mark Angle, then used to rotate a point, segment, circle, or polygon interior. Point rotations can drive iteration. Its value stays fixed within each iteration sequence and follows source edits. Select a measured angle or arc angle and use Number → Calculate with `v/2`, `v+10°`, or `v/90°`. Degree results can be marked with Mark Angle and used for point rotation; the last expression produces a scalar ratio. Moving the measured geometry updates its calculations and rotation.

Scalar sources also support calculation expressions using `v`: point ratios, slopes, Value of Point, x/y/coordinate-distance measurements, and length ratios. A scalar calculation may control segment iteration depth when its truncated value is from 1 to 128.

Length and area measurements also accept expressions using `v`, such as `v*1.5+v/2`, `v^2` for a squared length, or `(v*v)/v`. Results preserve compatible units; length results can supply Mark Distance. Circle and polygon areas support scaling and chained calculations. Square roots halve even unit exponents (`sqrt(area)` returns cm); roots of lengths or angles return scalar values. Fractional dimensional powers such as `area^0.5` return scalar values, matching native Sketchpad. Integer literal powers and products/quotients support higher units such as cm⁴ and °², including chained roots. Source-dependent, computed, grouped, and negative exponents produce scalar results, matching native Sketchpad. Results follow source edits and can feed marked-distance and live-depth iteration. Source-dependent powers can supply scalar live depth or numeric iteration; invalid domains suppress dependent geometry. See [exponent evidence](dev/source_exponents_analysis.md). See [native evidence](dev/dimensional_roots_analysis.md).

For a point-only iteration, select the free seed point, its directly translated or rotated image, and a scalar parameter or calculation, then use Iterate. The value controls depth dynamically, with positive fractions truncated: 5.5 produces five steps. A calculated-angle orbit can use `360°/n` for rotation and `n-1` for depth without a connecting segment.

To carry a ray through iteration, select the free seed, its directly translated or rotated image, their ray, and a scalar parameter or calculation for depth. Fixed-angle and calculated-angle rotations update the entire ray orbit when their center, seed, or numeric source changes.

To dilate by a numeric value, select a scalar parameter, measurement, or calculation and choose Transform → Mark Ratio. Select a source point, segment, circle, or polygon interior and a center, then use Dilate with the marked ratio. Editing the numeric source updates the result. A directly dilated point can drive iteration; its numeric ratio stays fixed within each sequence. Negative ratios reverse sides; zero ratios collapse geometry, with zero-radius circles omitted.

Point-to-segment, ray, and line distance uses the perpendicular distance to the supporting line, matching native Sketchpad even beyond an endpoint. Select the point and straight object in either order and choose Measure > Distance. The value stays live, supports decimal precision 0–5, and can feed length calculations and marked-distance iteration. See [native evidence](dev/point_line_distance_analysis.md).

Measure > Equation accepts supported original, segment-radius, and transformed circles in a square Cartesian coordinate system. Their equations follow changes to the circle, its transformation inputs, and the frame. See [native circle-equation evidence](dev/circle_equation_hosts_analysis.md).

Measure > Equation also accepts parallel/perpendicular infinite lines and coordinate axes. Line equations switch between sloped and vertical forms after edits; coordinate axes display `y = 0.00` and `x = 0.00`. Segments, rays, and angle bisectors are not eligible. See [native line-equation evidence](dev/constructed_line_equations_analysis.md).

`cargo run --bin gsp_probe -- coverage ../gsp-rs` recursively checks `.gsp`
files for byte-preserving saves and reports object decoding and partial-view
counts, with unsupported object counts grouped by native code. Passing byte
preservation alone does not establish editing support or command parity.
The current 1,599-file corpus scan decodes all 285,703 object slots, including
all 5,356 embedded-tool prototype slots, and round-trips every original
byte-exactly. This is an import-coverage result; structurally partial documents
and incomplete commands remain tracked in [COMMAND_PARITY.md](COMMAND_PARITY.md).
Of the 103 isolated embedded-tool views, 95 are now structurally complete; the
remaining eight contain read-only image resource tables.

Loci retain defined samples when their targets become undefined at some
positions. Point curves break at missing samples; object families omit those
members, and polygon sweeps leave gaps instead of connecting across them.
Imported parameter curves retain their native sample counts, including counts above 4,096, and accept alternate dependency ordering. Locus dependencies can pass through iteration nodes and transformed polygons. Calculations derive their units from the current expression and operands when a saved header retains an older quantity category.
Geometric loci driven by points on rays and lines retain their saved finite sampling intervals, including negative line parameters. Sampling, curve points, and copied constructions use those intervals.

Disk and sector transforms use the shared affine evaluator and editor commands. Their fill styles, drawing order, picking, and SVG output follow the transformed geometry. Coordinate helpers serialize their own references, allowing copied constructions to retain partial coordinate systems and labeled unit points.

Geometric-driver loci reuse shared affine/Colorized Segment, Ray and Line targets and generic finite Segment hosts. Evaluated samples carry their own pen/color to SVG, canvas and picking. Current evidence and supported boundaries are recorded in [the implementation notes](dev/locus_generic_straights63_implementation.md); corpus decoding does not establish complete command parity.

Function plots retain native six-digit sample counts while canvas, SVG, picking,
and point-curve evaluation continue to use bounded caller-selected resolutions.
Mapped iterations can present a carried function plot as its own styled binding,
reevaluating the function and coordinate frame in every mapped scope. Native and
port evidence is recorded in [the function-plot completion analysis](dev/function_plot_completion_analysis.md).

Imported general mapped iterations retain fixed depths through 4,096, matching
the existing live-depth evaluator and its independent per-level and total-state
resource guards. Desktop authoring and the specialized iteration commands keep
their 1–128 limit. This restores the native 179-step protractor and the
800-step Collatz construction; see
[the high-depth iteration analysis](dev/high_depth_mapped_iteration_analysis.md).

Referenced captions can use independent and sampled pictures as live Value
sources, retaining Native's localized generated picture name. The legacy
Chinese short-header caption profile is decoded with its GBK text cohort, so
mixed calculation/text captions such as the clock read `4时19分41秒`. See
[the referenced-caption completion analysis](dev/referenced_caption_completion_analysis.md).

Imported code-26 custom polar translations treat their angle slot as a
contextual degree magnitude. This includes native calculations reported as
Scalar, matching the shared transform model's existing numeric-operand rule;
their distance slot continues to interpret Length or Scalar values as signed
centimeters. Sampled mappings of colorized outputs retain each evaluated
spectrum color in canvas and SVG strokes. See
[the scalar-angle analysis](dev/scalar_angle_polar_translation_analysis.md).

Native sampled custom transformations of pictures retain their source pixels
and validated point-mapping construction. Bounded textured meshes drive canvas
rendering, picking, and triangulated SVG export while preserving native picture
order and sampling metadata. Their Drawing Properties can edit the sample budget
and Native's four allowed-distortion levels in a verified transaction. The wire
evidence and admission boundary are in
[the sampled-picture analysis](dev/sampled_transformed_pictures_analysis.md).

Imported Animation buttons retain the observed high-rate parameter-reset
settings and the legacy clock variant with its short header and code-page-936
label. Point rates remain separately bounded. Imported bidirectional settings
ignore an inapplicable stored Once bit, matching Native, and zero-rate members
of multi-target buttons remain stationary while the other members animate.
Newly authored buttons retain Native's stricter positive-rate and Once rules.
See [the compatibility analysis](dev/legacy_animation_settings_analysis.md).

Imported loci also retain native sparse or parallel-branch dependency captures
and can apply an explicit stored path transiently to an existing point when
that path is independent of the point's saved construction. This includes
constrained, transformed, path-value, iteration-alias, and colorized points;
ordinary loci whose driver already uses their stored path retain the original
ordering rules. Filled-region locus targets share the disk, sector,
circular-segment, and polygon sampler; point-output sampled objects can serve
as bounded outer curve targets. The measured corpus effects and strict
admission rules are recorded in [the original locus analysis](dev/locus_midpoint_analysis.md)
and [the explicit-path follow-up](dev/transient_locus_drivers_analysis.md).

Imported live captions understand the remaining observed native symbol-font
tokens, signed numeric terms, overbars/arrows/arcs, vertically centered groups,
outer frame metrics, and source-specific names, definitions, expressions, and
geometry values. Legacy Chinese markup is decoded strictly as GBK when the
complete markup is not UTF-8. These forms use the existing rich-text tree and
live resolver; frame metrics remain non-text presentation metadata. This
bounded expansion lowers unsupported corpus slots from 3,730 to 3,471 while
preserving all 1,599 source files byte-for-byte. The 14 remaining referenced
captions are guarded by parents whose object kinds are not yet supported.

Imported parameter-driven loci can now sweep a live segment, filled region, or
nested point-curve collection instead of being limited to a single point trace.
The same scoped numeric evaluator drives canvas/SVG geometry, picking, range
edits, and newly authored subtype-42/43 records. In addition to 67 geometric
families driven directly by parameters, it now accepts the 22 observed scalar
or degree calculation drivers and restores 19 descendants. Unsupported corpus
slots fall from 3,471 to 3,363; all-kind-complete documents rise from 1,228 to
1,243 and drawing-complete documents from 1,258 to 1,273. All 1,599 originals
remain byte-exact.

Radius measurements now use one round-geometry path for circles, circular arcs,
and sectors. The six native arc/sector measurements restore 91 already-supported
descendants, reducing unsupported slots from 3,363 to 3,266. Fully decoded and
drawing-complete document counts rise to 1,248 and 1,278 respectively; all 1,599
source files remain byte-exact. The editor can author Radius for supported arcs
and sectors on single- or multi-page sketches.

Ratio measurements now also decode and author the native two-segment form. Its
live scalar value is the first segment's length divided by the second's, and it
updates through ordinary or transformed segment geometry. Eight corpus roots
and 68 descendants restore 76 drawing slots: unsupported slots fall from 3,266
to 3,190, while all-kind-complete and drawing-complete documents rise to 1,253
and 1,283. All 1,599 originals remain byte-exact.

Sector interiors now accept the native name/style metadata already treated as
inert presentation data for circle interiors. This restores 16 drawing sectors,
12 embedded-tool sectors, and 51 drawing descendants without exposing the
metadata as an editable geometry label. Unsupported slots fall to 3,111;
all-kind-complete and drawing-complete documents rise to 1,256 and 1,286, and
all source files remain byte-exact.

Circular-segment interiors now preserve both observed presentation profiles and
share analytic Area and Perimeter evaluation through their source arc and closing
chord. Six interiors, two Area values, and three descendants restore 11 drawing
slots. Native Arc Length values that reference a circle followed by two endpoint
directions are also live and authorable; 12 roots restore 61 descendants. Together
these passes lower unsupported slots from 3,111 to 3,027, raise complete documents
to 1,260 and complete drawings to 1,290, and retain byte-exact round trips for all
1,599 originals.

Area, Perimeter, and Arc Length imports now recognize transformed and Colorized
sector hosts through the shared sector geometry, restoring six drawing slots.
Animation buttons targeting transformed points use a temporary evaluator override:
their dependents move during playback, but the underlying construction and saved
bytes do not change, and Stop restores the constructed position. This admits 32
native buttons. Together the two passes lower unsupported slots from 3,027 to
2,989, raise complete documents to 1,262 and complete drawings to 1,292, and
lower partial views to 869. All 1,599 originals remain byte-exact.

Native code-121 segment congruence and direction markers import, render, pick,
copy, and save on supported original or transformed segments. One-, two-, and
three-tick marks use native fixed dimensions; forward and reverse arrows use
the stored host direction. The final imported direction-arrow profile also
follows a supported sampled point curve and uses its evaluated local tangent;
that curve-host form remains import-only. All forms were verified against
Sketchpad 5 under Xvfb/Wine. See
[the sampled-curve evidence](dev/sampled_curve_arrow_analysis.md).

Legacy code-119 angle rotations now share the live numeric-rotation model for
strict point, segment, and analytic-arc groups. The seven admitted roots retain
their original bytes and restore three dependents, lowering unsupported slots to
2,952; six roots whose source family is not yet represented remain guarded.
The original 16-page example also opened and resaved byte-identically in
Sketchpad 5 under Xvfb/Wine.

Legacy code-125 and code-124 coordinate values now evaluate as radius and polar
angle for a supported point/frame pair, regardless of the displayed grid form.
The angle readout follows the document's degree/radian setting. Two roots and
six dependent calculations recover, lowering unsupported slots to 2,944 while
all 1,599 originals remain byte-exact. A port-authored pair opened and resaved
byte-identically in Sketchpad 5 under Xvfb/Wine.

Fixed rectangular translations now retain native centimeter or inch units on
each axis. This restores ten visible Segment translations, lowering unsupported
slots to 2,934. A port-authored half-inch by negative-quarter-inch translation
displayed at the expected 48-by-24-pixel offset and resaved byte-identically in
Sketchpad 5 under Xvfb/Wine.

The same code-17 payload also uses unit tag 0 for literal pixel displacement.
Seven path-point images in the multi-page teaching sample now decode, lowering
the latest unsupported total from 2,830 to 2,823. A port-authored `(120, −80)`
pixel translation opened in Sketchpad 5 and displayed exactly 120 pixels right
and 80 pixels down under Xvfb/Wine.

Native code-19 class-13/subtype-69 circular-segment translations now retain their
live marked horizontal distance and fixed vertical offset. Both 22-object chains
in the `方轮小汽车` samples decode, render, pick, and measure as arc-plus-chord
regions, restoring 44 drawing slots and lowering the unsupported total to 2,890.
A port-authored pair displayed at the expected offset in Sketchpad 5 under
Xvfb/Wine.

Fixed rotation, reflection, and fixed dilation now use that same distinct
circular-segment geometry. Fourteen ready native roots restore 37 further drawing
objects, lowering the current unsupported total to 2,853. Sketchpad 5 rendered a
port-authored source with its 180-degree rotation, reflection, and half-scale
dilation at the expected positions under Xvfb/Wine.

The ordinary x-axis can now be the source of the two rotation forms observed in
native files: code-27 fixed angle and code-29 numeric angle. The axis remains an
infinite line through live evaluation, editing, and save/reopen. Two roots and
21 descendants restore 23 drawing slots, lowering unsupported objects to 2,830;
fully decoded and drawing-complete documents rise to 1,272 and 1,302. A
port-authored fixed/numeric pair opened and rendered correctly in Sketchpad 5
under Xvfb/Wine. Other axis transforms remain guarded.

Two legacy 16-byte code-10 disk headers now retain their live Circle sources,
RGB fill, and native implicit 75% opacity. The original clock renders the two
overlapping filled disks in Sketchpad 5, both files remain byte-exact, and the
latest unsupported total falls to 2,821.

Literal code-123 points now follow ordinary point-output Locus paths at their
stored normalized fraction. Four roots and 72 descendants restore 76 drawing
slots, lowering unsupported objects to 2,745 and raising fully decoded/drawing-
complete documents to 1,274/1,304. A port-authored `2/3` midpoint-locus point
rendered correctly in Sketchpad 5 under Xvfb/Wine, whose Ctrl+S save remained
byte-identical. Parameter-locus and FunctionPlot literal forms remain guarded.

Native fixed-coordinate points now also accept a live Degrees value and use its
displayed magnitude as the graph coordinate, matching the established numeric
XY rule. Three code-110 roots and 82 descendants restore 85 drawing slots;
unsupported objects fall to 2,660 and fully decoded/drawing-complete documents
rise to 1,276/1,306. Length and area sources remain guarded.

Legacy Cartesian coordinate-pair readouts now preserve Native's alternate
two-byte rich-text cache tail, using the same bounded size rule as equation
readouts. Twenty ready code-49 roots and one descendant restore 21 drawing
slots; unsupported objects fall to 2,639 and complete documents/drawings rise
to 1,277/1,307. The remaining code-49 object still has an unsupported point.

Legacy Value of Point measurements now project supported points onto live Locus,
ParameterLocus, FunctionPlot, and sampled-mapping paths; matching points on sector
boundaries retain their exact perimeter parameter. Twenty-two code-94 roots and
143 dependents restore 165 drawing slots. Unsupported objects fall to 2,474 and
complete documents/drawings rise to 1,285/1,315, with all 1,599 originals still
byte-exact. A newly authored sector-boundary value file also opened and saved
unchanged in the supplied Windows Sketchpad 5 under Xvfb/Wine.

Anchored referenced labels now use the shared rich-caption source rules instead
of a second, narrower whitelist. This restores coordinate-pair and nested
point-label values, including the observed calculation-cache `?3`/`?4`
containers. Thirty-two code-90 drawing slots are recovered: unsupported objects
fall to 2,442 and complete documents/drawings rise to 1,305/1,335, while all
1,599 originals remain byte-exact.

Mapped iterations now carry `ParameterLocus` point traces through the same
scoped collection evaluator as ordinary point loci. Ten code-89 roots, 25
bindings, and eight dependents restore 43 drawing slots; unsupported objects fall
to 2,399 and complete documents/drawings rise to 1,314/1,345. A hidden-source
spiral control displays the bound traces in both Native Sketchpad 5 and the port,
and Native saves the 4,532-byte control unchanged.

Mapped iterations now retain hide/show, animation, and movement buttons as
capture-only dependency nodes without treating actions as repeatable bindings.
Twenty-one code-89 roots, 147 code-77 bindings, and eight dependents restore 176
drawing slots; unsupported objects fall to 2,223 and complete
documents/drawings rise to 1,326/1,358. With every ordinary source hidden, a
five-binding control displays the recovered red point and green arc sequences in
both Native Sketchpad 5 and the port, and Native saves all 5,780 bytes unchanged.

Imported mapped-iteration value tables now keep all carried numeric/function
columns, their screen anchor, caption and font, and evaluate the initial value
plus each numbered generation. The `N!.gsp` Native control and port SVG both
produce rows 0–5 ending in `7.00` and `5040.00`. This restores all 42 remaining
code-92 drawing records: unsupported slots fall to 2,181 and complete
documents/drawings rise to 1,346/1,381, while all 1,599 originals remain
byte-exact.

General native snapshot tables preserve two to seven source columns, rich
headers, stored rows, anchors, captions, fonts, visibility, and alignment bytes.
Native and port checks cover both a nine-row coordinate history and a zero-state
π-estimation table with an undefined cached cell. This restores all 40 code-91
drawing records: unsupported slots fall to 2,141 and complete
documents/drawings rise to 1,358/1,393, with all 1,599 originals still
byte-exact.

Strict imported code-26 polar translations now cover native Point and Segment
outputs with exact Degrees/Length operands, including labelled objects and
points later consumed by sampled mappings. Native Xvfb/Wine checks cover free,
nested, trace-linked, and Segment examples; all tested saves are byte-exact.
This restores 95 slots, lowering unsupported objects to 2,046 and raising
complete documents/drawings to 1,369/1,404. Scalar-operand and embedded-tool
variants were kept separate at that checkpoint.

Plot Value points now accept every decoded numeric quantity and retain its unit
for presentation while using the displayed magnitude on axes and geometric
paths. Function plots consume the live value as x; sector boundaries use their
normalized physical perimeter for both live code-95 and literal code-123
points. Native accepted freshly authored quantity-axis and sector controls
byte-for-byte. This restores 90 slots, lowering unsupported objects to 1,956
and raising complete documents/drawings to 1,380/1,415.

Mapped-iteration import now canonicalizes repeated IDs in Native's carried-DAG
tail while preserving the original wire packet. The evaluator still receives
one ordered instance of each construction, and the public authoring API retains
its unique-carried-input guard. This restores 16 code-89 definitions and 59
bindings, lowering unsupported objects to 1,881 and raising complete
documents/drawings to 1,391/1,426. A recovered tiling sketch matches Native's
iterated geometry, and Native saves all 8,184 source bytes unchanged.

Mapped iterations can now capture an earlier complete mapped root and its
structural bindings. The bindings remain scope descriptors rather than repeated
output values, and import requires every inner construction that depends on an
outer seed. This restores 11 code-89 definitions and 48 bindings, lowering
unsupported objects to 1,822 and raising complete documents/drawings to
1,401/1,436. Native and the port agree on the 3/9/18/30/45 pipe progression;
Native preserves the 5,682-byte control exactly.
