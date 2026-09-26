# Sketchpad RS

Experimental Linux editor for Sketchpad 5 `.gsp` documents. It uses winit and egui for the window and controls, and wgpu with a WGSL shader for sketch geometry.

## Run

```sh
nix develop -c cargo run --bin sketchpad_rs -- path/to/sketch.gsp
```

The path is optional. The Nix shell supplies the desktop libraries and fontconfig needed at runtime. Use the File menu to create, open, save, close, and quit; Ctrl+S saves the current file and Ctrl+Shift+S opens Save As. New, Open, Close, and Quit prompt about unsaved changes. File > Document Options adds, copies, renames, removes, and reorders pages in supported sketches; Undo and Redo handle these changes. Page removal asks for confirmation. The File menu imports a single PNG picture, exports the visible page as SVG or PDF, and sends it to the Linux default printer. PDF output requires `rsvg-convert`; printing also requires `lp`. The toolbar creates points, segments, circles, and literal text. Clicking a supported segment, ray, line, circle, or arc with the Point tool creates a point constrained to that object, including constructed straight paths and transformed segments/circles. Value of Point measurements on these paths can feed live calculations and iteration depth. Select the required objects for the Construct commands. The Measure menu adds live measurements, including length, two-point and point-to-straight-object distance, area, perimeter, point value on a segment, ray, line, circle, or arc, signed point ratio, and slope of straight objects (including constructed lines and transformed segments) in a Cartesian coordinate system. The Number menu supports parameters, source-free calculations using numbers, decimals, arithmetic, powers, functions, π, and e, live multiplication of a selected length measurement or length calculation by 0–99 or division by 1–99, arithmetic on two selected lengths using a + b, a - b, a * b, or a / b in selection order, and live expressions of a selected numeric parameter or a prior parameter calculation, plus parameter tables with row controls. Number > Attach Selected Value to Point creates a live numeric label for a selected point and parameter or calculation. To plot a point in a supported one-page sketch, define a coordinate system, then choose Cartesian or polar Plot Points from Graph and enter coordinates. The dialog can plot successive points without selecting a table. Display changes supported point styles, line widths and patterns, colors, and text font families; it also traces selected points and segments while dragging and clears the painted trails with Erase Traces. Edit > Cut, Copy, and Paste use an internal sketch clipboard for supported objects, and Copy and Paste also work for verified constructions across supported multi-page pages; Edit > Clear or Delete removes selected objects and their dependents in supported single-page files and supported multi-page construction and measurement groups. The Transform menu can translate or rotate a point, reflect a point or circle across a selected line, dilate a point or circle around a selected center, and iterate a selected free point with its directly translated, rotated, dilated, or reflected image to a chosen depth from 1 to 128, optionally carrying the segment between them. Selecting the connecting ray and a scalar parameter as well makes the iteration depth follow the parameter. To iterate a numeric value along with the point, select the seed point, transformed image, parameter, direct parameter calculation, and numeric label attached to the image, then choose Iterate. Selecting a second scalar parameter makes its value control the numeric iteration depth. To branch a triangular interior, select its free seed, two directly translated image vertices, the interior, and optionally its sides and a scalar depth parameter. Drag a point to update its dependents or its position on a segment, ray, line, circle, or arc. Scroll over the canvas to zoom. Multi-page files have page controls in the left panel. Saving preserves the selected page for reopening, including navigation through Page Link buttons. The Point, Segment, and Circle tools, plus the Construct commands for lines, rays, midpoints, parallel and perpendicular lines, straight and circle intersections, angle bisectors, three-point arcs, arcs on circles, polygon interiors, and circles with a segment radius, work on supported multi-page pages. Points can be constrained to newly created paths. Supported multi-page pages can also add length, distance, perimeter, circle, polygon, arc, angle, radius, and point-ratio measurements. Existing points can be moved, and changes to existing object styles and trace flags can be saved. Literal captions can also be created, edited, moved, formatted, copied between supported pages, and cleared; mixed character formatting and font families survive saving. See [multi-page text checks](dev/multi_page_text_analysis.md).

Graph also offers Square, Polar, Rectangular, and trigonometric X-axis grid forms, Show Grid, Dotted Grid, and Snap Points for a defined coordinate system. A one-distance frame can switch to Rectangular and drag its vertical unit independently; a two-distance frame can switch between Rectangular and Polar. Graph > Plot Value on Axis/Path accepts a number in its dialog, or plots a live value when t₁ is selected. Select a segment, ray, line, circle, or axis to use it; otherwise the active X axis is used. Segment values run from 0 to 1, ray values start at 0, and circle values count counterclockwise turns from the rightmost point.
Selecting a point before Graph > Define Coordinate System uses it as the origin, including when the sketch already has a frame. Selecting a circle defines a square unit frame centered on that circle, with the circle's radius as one unit. Select one segment or distance measurement to define a square unit scale, or two in selection order to define separate horizontal and vertical scales; a selected point can set the origin. Select an origin, unit point, axis, or grid intersection and choose Graph > Mark Coordinate System to make that frame active for subsequent plots and coordinate measurements. Snap Points leaves the visible grid on screen.
When a sketch has multiple frames, the editor and SVG export draw each visible frame using its own Square, Rectangular, or Polar grid form and dotted setting.
The Number menu can create t₁ and t₂ parameters or multiple named ASCII parameters in a supported single-page sketch, and edit finite values with up to 32 native numeric tokens. Parameter captions support 0–5 decimal places and use scientific notation at magnitudes greater than one million; calculations retain the full value. Decimal edits increase display precision up to five places. Supported values can be tabulated into tables with snapshot row controls; tables remain limited to −99.99 through 99.99 with two decimal places. The default one-column t₁ table caches values from −99.99 to 99.99; fractional, negative, two-digit, and later-source snapshot files cross-opened in Sketchpad 5. The supported two-column table uses t₁ and t₂ values from −99.99 to 99.99 with up to 256 rows; generated five-row and mixed-width files opened and saved byte-identically in Sketchpad 5. Graph > Plot Table Data plots its Cartesian or polar rows, including distinct and fractional values.
Display > Animate Objects runs a selected parameter over its stored domain or animates a supported point. Selecting a construction such as a segment or circle animates its source points, even when those points are hidden. Edit > Action Buttons > Animation creates a clickable motion button for selected parameters and points, or for a selected segment, line, ray, circle, or polygon with up to seven vertices through its source points. Animation buttons for supported points and segment sources can also be created on existing multi-page sketches. Parameter buttons can use continuous or discrete changes with an editable rate, step, and domain. The Motion Controller pauses, resumes, changes speed, and stops Display animation.
Select a caption and highlight characters in the sidebar text field to apply Bold, Italic, Underline, Larger, Smaller, or a font family to that range. These mixed-format captions render in the editor and SVG, and survive word edits, move, and copy/paste. Text insertions inherit the neighboring style, and replaced text inherits the first replaced character's style.
Polygon interiors can be reflected, rotated by a fixed or marked angle, dilated by a fixed or marked ratio, translated by fixed or marked polar and rectangular distances or a marked vector, and measured for live area. These results update when their source vertices or defining points move, and the generated files have been opened and resaved in Sketchpad 5.

Select standard length, distance, perimeter, circumference, radius, area, angle, arc, ratio, x/y coordinate, coordinate distance, slope, or Value of Point measurements and use **Measure → Decimal Places** to choose 0–5 decimal places. Precision is saved separately for each measurement and preserved by undo and copy/paste. Supported single-page calculation captions also accept 0–5 decimal places, including literal formulas, source-dependent formulas, measured-length arithmetic, and two-length arithmetic. Mixed selections change together with undo and copy/paste support. Calculations keep using the full measured values. The same precision appears in desktop captions and SVG exports, including measurements on supported multiple-page sketches.

Selected numeric values also support `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `abs`, `sqrt`, `ln`, `log`, `sgn`, `round`, and `trunc` in calculation expressions, including nested calls. Trigonometric arguments use degrees; inverse functions accept scalar arguments and return degrees usable as marked rotation angles. Constants `pi` (or `π`) and `e` can appear alongside the selected source. With no source selected, the calculator accepts the same functions and constants, including signed numbers and degree literals. Scalar formulas can supply iteration depth; degree formulas can supply marked rotation angles. `log` uses base 10. `abs` preserves the source’s length, area, or angle units. The remaining non-trigonometric functions currently require scalar arguments. Undefined results stop dependent geometry until the source is valid again.

Numeric iterations can use a formula with multiple inputs. Select the free point, its image, the parameter to iterate, the scalar formula, and the formula’s label attached to the image. Other formula inputs stay at their current values during the iteration. Editing an external input refreshes the whole sequence. For example, `k+x` advances by `k` on each step when `x` is the mapped parameter.

Select two to sixteen numeric measurements, parameters, or calculations to combine them in one formula. Use `v1`, `v2`, … in selection order; `a` and `b` also refer to the first two. Examples include `(a+b)/2` for two lengths, `v1*v2` for a length scaled by a parameter, and `sqrt((v1+v2)*v3)` for three scalars. Results retain compatible units and update when any source changes. Scalar results can control iteration depth; degree results can control rotation. Expressions remain limited to 32 native tokens.

To carry a circle through an iteration, select a free center point, its directly translated, rotated, dilated, or reflected image, and the circle centered on the seed through that image. Iterate uses the dialog depth (1–128); also select a scalar parameter or calculation to control depth live. Calculated rotation angles such as `360°/n` and a depth such as `n-1` update the circle orbit together. Repeated circles retain their binding’s stroke style and can be selected or hidden. Dilated mappings use a fixed scale, a marked three-point ratio, or a marked scalar parameter/measurement/calculation. Centers and ratio inputs can be supported constructed points. Repeated circles scale their radii with each step; negative factors alternate sides of the center. The same point mappings support point, segment, ray, circle, and combined point/parameter iterations. Reflected mappings alternate across the current mirror. Vector mappings update with their endpoints; using the seed as the vector start fixes later points at the endpoint, while using it as the endpoint doubles the displacement on each step. Marked polar and rectangular distances, either mixed rectangular translation, and three-point marked angles also support iteration. Length calculations can supply the marked distances. Every direct seed reference is substituted at each step, including marked angle and ratio inputs; external construction inputs stay fixed within each sequence.

For a small interoperability sample, run `cargo run --bin gsp_probe -- demo /tmp/sketch.gsp`.

Removing a page leaves Page Links to that page visible with an unavailable destination, as in Sketchpad 5. The confirmation names affected links.

## Current scope

Construct > Locus accepts a selected point constrained to a segment, circle,
or arc and a dependent point, segment, ray, line, circle, arc, or polygon
interior, including an interior translated by a marked vector. Point loci
follow supported dependency graphs; other loci draw sampled object families.
The line targets include lines constructed
parallel or perpendicular to another line; angle bisectors produce ray
families.

Display > Animate Objects moves selected free points or points constrained to
segments, rays, lines, circles, or arcs and updates their dependent constructions. Free
points wander within the visible canvas; segment and arc points reverse at
their endpoints, ray and line points reverse at the visible path ends, and circle points loop. The Display menu and Motion Controller
can change speed, pause, resume, and stop the motion. Motion keeps the point's
current position when stopped and can be undone as one edit.

Edit > Action Buttons > Hide/Show creates one button for all selected supported
objects, including on an existing supported multi-page page. Clicking it hides
or shows those objects together.

Edit > Action Buttons > Animation creates one button for the selected animatable
points, including on a supported existing multi-page page. The dialog sets
direction, speed, and Once Only for each point. Clicking the button starts or
stops their motion together.

Edit > Action Buttons > Movement takes a selected free point followed by a
destination point, including on a supported existing multi-page page. Its
dialog sets Slow, Medium, Fast, or Instant speed and whether to follow a moving
destination. Clicking the button moves the source point; clicking again stops
an active movement.

Edit > Action Buttons > Scroll takes one selected point, including on a
supported existing multi-page page. Its dialog places the point at the canvas
corner or center when the button is clicked.

Edit > Action Buttons > Presentation takes selected action buttons in selection
order. Its dialog chooses simultaneous or sequential activation. Sequential
presentations wait for a running action to finish before advancing.

Edit > Action Buttons > Link creates a URL button or, on any page of a supported
multi-page document, a Page Link. The editor opens HTTP(S) and file links with
the system handler, opens `sketchdoc://` links in the current document's folder,
and follows Page Links within the document. A Page Link can also activate a
Hide/Show button on its destination page. The resulting Hide/Show state can be
saved in a supported multi-page document. The destination button and Page Link
can be created together before the first save.

The writer supports the constructions and measurements listed in [COMMAND_PARITY.md](COMMAND_PARITY.md), including polygons, signed point ratio, slope, Cartesian arithmetic and sine/cosine/tangent function plots, rectangular and polar parametric plots from two functions, editable plot domains and sample counts, a single t₁ point plotted in Cartesian or polar coordinates, Hide/Show, Animation for up to seven verified source points including mixed free, segment, and circle points, one-pair point Movement, one-point Scroll, Presentation, URL Link, and Page Link action buttons on supported multi-page documents, selected-object styling, and bounded parameter/table workflows. It can update coordinates, supported point constraints, visibility, and styles in place in existing documents, and remove selected objects with their dependents in supported single-page documents. Copy includes a selected object's construction parents; Paste duplicates them with a 20-unit offset. Transform > Mark Center and Mark Mirror let a later rotation, dilation, or reflection use one selected source object. It preserves unsupported source records byte for byte. Structural edits to a document with unsupported content are refused when saving. Saves write through a temporary file in the destination directory and rename it into place.

Other Sketchpad features are incomplete. In particular, many transformations, graphing commands, advanced image and table workflows, custom tools, action button variants, other import formats, page setup, print preview, and full multi-page editing are still pending. See [COMMAND_PARITY.md](COMMAND_PARITY.md) for the command inventory. This build is not ready as a Sketchpad 5 replacement.

## Source layout

- `src/document.rs` holds the document data model and basic object creation.
  `src/document/` groups editing operations by responsibility: geometry,
  measurements, coordinates, functions, iterations, text, tables, actions,
  selection, pages, and history. File handling lives in `records.rs`,
  `decode.rs`, `encode*.rs`, `validation.rs`, and `save.rs`.
- `src/main.rs` starts the desktop editor. `src/editor/mod.rs` holds editor
  state; `runtime.rs` owns the window and render loop. Commands, picking,
  animation, file operations, and fonts have separate modules.
- `src/editor/ui.rs` coordinates menus, panels, canvas, and dialogs in render
  order. Their implementations live in `menu.rs`, `panels.rs`, `canvas.rs`,
  and the `*_dialogs.rs` modules.
- `src/document/tests/` and `src/editor/tests/` group the existing unit tests
  by feature. Binary fixtures remain in `tests/fixtures/`.

## Verification

Run `cargo test`. Tests cover record parsing and byte preservation, dependency updates, geometry, undo/redo, and save/reopen. All 1,599 `.gsp` files in the sibling project and the bundled 523-object Example Tools document pass unedited byte-preservation verification; many remain partial views and are guarded against unsafe edits. Generated constructions, measurements, text, action buttons, styles, bounded number/table files, and arithmetic and sine/cosine/tangent plots have been opened and saved in the supplied Windows Sketchpad 5 through Wine. Native KDE screenshots confirm geometry and style rendering; full command and interaction verification remains open.

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

Loci retain defined samples when their targets become undefined at some
positions. Point curves break at missing samples; object families omit those
members, and polygon sweeps leave gaps instead of connecting across them.
