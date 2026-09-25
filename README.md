# Sketchpad RS

Experimental Linux editor for Sketchpad 5 `.gsp` documents. It uses winit and egui for the window and controls, and wgpu with a WGSL shader for sketch geometry.

## Run

```sh
nix develop -c cargo run --bin sketchpad_rs -- path/to/sketch.gsp
```

The path is optional. The Nix shell supplies the desktop libraries and fontconfig needed at runtime. Use the File menu to create, open, save, close, and quit; Ctrl+S saves the current file and Ctrl+Shift+S opens Save As. New, Open, Close, and Quit prompt about unsaved changes. The File menu imports a single PNG picture, exports the visible page as SVG or PDF, and sends it to the Linux default printer. PDF output requires `rsvg-convert`; printing also requires `lp`. The toolbar creates points, segments, circles, and literal text. Clicking a segment, circle, or arc with the Point tool creates a point constrained to that object. Select the required objects for the Construct commands. The Measure menu adds live measurements, including length, distance, area, perimeter, signed point ratio, and slope in a Cartesian coordinate system. The Number menu supports a default parameter, a literal calculation, and a one-column parameter table with row controls. To plot a point in a supported one-page sketch, define a coordinate system, then choose Cartesian or polar Plot Points from Graph and enter coordinates. The dialog can plot successive points without selecting a table. Display changes supported point styles, line widths and patterns, colors, and text font families; it also traces selected points and segments while dragging and clears the painted trails with Erase Traces. Edit > Cut, Copy, and Paste use an internal sketch clipboard for supported objects; Edit > Clear or Delete removes selected objects and their dependents in supported single-page files. The Transform menu can translate or rotate a point, reflect a point or circle across a selected line, or dilate a point or circle around a selected center. Drag a point to update its dependents or its position on a segment, circle, or arc. Scroll over the canvas to zoom. Multi-page files have page controls in the left panel and can save moved free points.

Graph also offers Square, Polar, Rectangular, and trigonometric X-axis grid forms, Show Grid, Dotted Grid, and Snap Points for a defined coordinate system. A one-distance frame can switch to Rectangular and drag its vertical unit independently; a two-distance frame can switch between Rectangular and Polar. Graph > Plot Value on Axis/Path accepts a number in its dialog, or plots a live value when t₁ is selected. Select a segment, ray, line, circle, or axis to use it; otherwise the active X axis is used. Segment values run from 0 to 1, ray values start at 0, and circle values count counterclockwise turns from the rightmost point.
Selecting a point before Graph > Define Coordinate System uses it as the origin, including when the sketch already has a frame. Selecting a circle defines a square unit frame centered on that circle, with the circle's radius as one unit. Select one segment or distance measurement to define a square unit scale, or two in selection order to define separate horizontal and vertical scales; a selected point can set the origin. Select an origin, unit point, axis, or grid intersection and choose Graph > Mark Coordinate System to make that frame active for subsequent plots and coordinate measurements. Snap Points leaves the visible grid on screen.
When a sketch has multiple frames, the editor and SVG export draw each visible frame using its own Square, Rectangular, or Polar grid form and dotted setting.
The Number menu can create t₁ and t₂ parameters or multiple named ASCII parameters in a supported single-page sketch, and edit values from −99.99 to 99.99 with up to two decimal places. Named parameters increase their display precision when a value needs a second decimal place and can be tabulated into tables with snapshot row controls. The default one-column t₁ table caches values from −99.99 to 99.99; fractional, negative, two-digit, and later-source snapshot files cross-opened in Sketchpad 5. The supported two-column table uses t₁ and t₂ values from −99.99 to 99.99 with up to 256 rows; generated five-row and mixed-width files opened and saved byte-identically in Sketchpad 5. Graph > Plot Table Data plots its Cartesian or polar rows, including distinct and fractional values.
Select a caption and highlight characters in the sidebar text field to apply Bold, Italic, Underline, Larger, Smaller, or a font family to that range. These mixed-format captions render in the editor and SVG, and survive word edits, move, and copy/paste. Text insertions inherit the neighboring style, and replaced text inherits the first replaced character's style.
Polygon interiors can be reflected, rotated by a fixed or marked angle, dilated by a fixed or marked ratio, translated by fixed or marked polar and rectangular distances or a marked vector, and measured for live area. These results update when their source vertices or defining points move, and the generated files have been opened and resaved in Sketchpad 5.

For a small interoperability sample, run `cargo run --bin gsp_probe -- demo /tmp/sketch.gsp`.

## Current scope

Construct > Locus accepts a selected point constrained to a segment, circle,
or arc and a dependent point, segment, ray, line, circle, arc, or polygon
interior, including an interior translated by a marked vector. Point loci
follow supported dependency graphs; other loci draw sampled object families.
The line targets include lines constructed
parallel or perpendicular to another line; angle bisectors produce ray
families.

Display > Animate Objects moves selected free points or points constrained to
segments, circles, or arcs and updates their dependent constructions. Free
points wander within the visible canvas; segment and arc points reverse at
their endpoints, and circle points loop. The Display menu and Motion Controller
can change speed, pause, resume, and stop the motion. Motion keeps the point's
current position when stopped and can be undone as one edit.

Edit > Action Buttons > Hide/Show creates one button for all selected supported
objects. Clicking it hides or shows those objects together.

Edit > Action Buttons > Animation creates one button for the selected animatable
points. The dialog sets direction, speed, and Once Only for each point. Clicking
the button starts or stops their motion together.

Edit > Action Buttons > Movement takes a selected free point followed by a
destination point. Its dialog sets Slow, Medium, Fast, or Instant speed and
whether to follow a moving destination. Clicking the button moves the source
point; clicking again stops an active movement.

Edit > Action Buttons > Scroll takes one selected point. Its dialog places the
point at the canvas corner or center when the button is clicked.

Edit > Action Buttons > Presentation takes selected action buttons in selection
order. Its dialog chooses simultaneous or sequential activation. Sequential
presentations wait for a running action to finish before advancing.

Edit > Action Buttons > Link creates a URL button or, on any page of a supported
multi-page document, a Page Link. The editor opens HTTP(S) and file links with
the system handler, opens `sketchdoc://` links in the current document's folder,
and follows Page Links within the document. A Page Link can also activate a
Hide/Show button on its destination page. The resulting Hide/Show state can be
saved in a supported multi-page document.

The writer supports the constructions and measurements listed in [COMMAND_PARITY.md](COMMAND_PARITY.md), including polygons, signed point ratio, slope, Cartesian arithmetic and sine/cosine/tangent function plots, rectangular and polar parametric plots from two functions, editable plot domains and sample counts, a single t₁ point plotted in Cartesian or polar coordinates, Hide/Show, one- or two-point Animation, one-pair point Movement, one-point Scroll, Presentation, URL Link, and Page Link action buttons on supported multi-page documents, selected-object styling, and bounded parameter/table workflows. It can update coordinates, supported point constraints, visibility, and styles in place in existing documents, and remove selected objects with their dependents in supported single-page documents. Copy includes a selected object's construction parents; Paste duplicates them with a 20-unit offset. Transform > Mark Center and Mark Mirror let a later rotation, dilation, or reflection use one selected source object. It preserves unsupported source records byte for byte. Structural edits to a document with unsupported content are refused when saving. Saves write through a temporary file in the destination directory and rename it into place.

Other Sketchpad features are incomplete. In particular, many transformations, graphing commands, advanced image and table workflows, custom tools, action button variants, other import formats, page setup, print preview, and full multi-page editing are still pending. See [COMMAND_PARITY.md](COMMAND_PARITY.md) for the command inventory. This build is not ready as a Sketchpad 5 replacement.

## Verification

Run `cargo test`. Tests cover record parsing and byte preservation, dependency updates, geometry, undo/redo, and save/reopen. All 1,599 `.gsp` files in the sibling project and the bundled 523-object Example Tools document pass unedited byte-preservation verification; many remain partial views and are guarded against unsafe edits. Generated constructions, measurements, text, action buttons, styles, bounded number/table files, and arithmetic and sine/cosine/tangent plots have been opened and saved in the supplied Windows Sketchpad 5 through Wine. Native KDE screenshots confirm geometry and style rendering; full command and interaction verification remains open.

`bash dev/native_desktop_smoke.sh path/to/sketch.gsp /tmp/sketchpad_capture` launches the built binary and captures the current desktop. When Nix is available, the script enters the development shell automatically to load its Wayland libraries.

`gsp_probe` provides headless `inspect`, `diff`, `verify`, `demo`, `replay`, and `svg` commands for comparing original and generated files and exercising document commands. Integrated Cartesian and polar t₁ Plot Points files were opened and saved byte-identically in Sketchpad 5 under Wine. The Ghidra scripts and current binary analysis are in [dev/ANALYSIS.md](dev/ANALYSIS.md).
