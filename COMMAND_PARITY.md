# Sketchpad 5 command parity

Source: the bundled Sketchpad 5 Reference Center menu pages. Status describes the current Rust port. **Partial** means a UI command exists but its full Sketchpad behavior and Windows cross-open have not yet been verified. **Pending** means absent.

The release gate is every sketch and document command verified after editing, saving, reopening, and cross-opening in Sketchpad 5. This gate is not met.

## File

| Command | Status |
| --- | --- |
| New Sketch | Partial |
| Open | Partial |
| Save | Partial: menu and Ctrl+S; an animation-stopped file saved from the desktop opened and resaved byte-identically in Sketchpad 5 |
| Save As | Partial: menu and Ctrl+Shift+S; broader document layouts pending |
| Close | Partial: unsaved-change dialog |
| Document Options | Pending |
| Page Setup | Pending |
| Print Preview | Pending |
| Print | Partial: visible page sent as PDF to the Linux default printer |
| Quit | Partial: unsaved-change dialog |
| Sketchpad LessonLink... | Outside release target (legacy service) |
| Sketchpad Sketch Exchange | Outside release target (legacy service) |

## Edit

| Command | Status |
| --- | --- |
| Undo | Partial |
| Redo | Partial |
| Cut | Partial: supported single-page objects and their dependents; save/reopen tested |
| Copy | Partial: selected supported objects and required parents, including segment and midpoint cases observed in Sketchpad 5; one- and two-column tables include their source parameters and embedded source IDs; copied function plots retain their frame style and definition, and derivatives retain their source definition. Copying a selected function includes its Cartesian plotted child; copying either parametric source includes both functions and the curve, as observed in Sketchpad 5 |
| Paste | Partial: internal clipboard, 20-unit offset, and generated segment cross-opened in Sketchpad 5; copied one- and two-column tables remap their source IDs and pasted number/table anchors have the native 20-unit offset. Generated copied function plot, parametric curve, and derivative files opened and saved byte-identically in Sketchpad 5, with separate formula positions visible. A native copied plot imports as editable; its copied frame origin was moved in Rust and saved byte-identically in Sketchpad 5. Pasted plots hide their copied frame parts. Both choices of a selected parametric source receive the native `f`/`g` names; other graph copies remain pending |
| Clear | Partial: selected objects and dependent constructions in supported single-page files; generated result reopened in Sketchpad 5. Saved trailing function, Cartesian and parametric plot, and plotted-point groups opened and saved byte-identically in Sketchpad 5 after removal. Independent free points before surviving function plots, parametric plots, plotted points, and one- or two-column tables, plus an independent caption before a function plot, can be deleted with references remapped. Deleting a parameter removes its dependent table and path point while preserving an unrelated frame. Other earlier object kinds remain guarded |
| Action Buttons | Partial: single- and two-target Hide/Show and Animation, one-pair point Movement, one-point Scroll, Presentation of selected action buttons, URL Links, and Page Links on supported multi-page files; broader targets pending |
| Select All | Partial |
| Select Parents | Partial: direct dependencies of supported objects |
| Select Children | Partial: direct dependents of supported objects |
| Crop Picture to Polygon | Pending |
| Split/Merge | Pending |
| Edit Definition | Pending |
| Properties | Pending |
| Preferences | Pending |

## Display

| Command | Status |
| --- | --- |
| Point Style | Partial: selected points, undo/redo, SVG/native rendering, save/reopen and generated file cross-open; future-object default pending |
| Line Style | Partial: width and solid/dashed/dotted patterns for supported strokes; generated segment/circle cross-opened; future-object default pending |
| Color | Partial: RGB for supported points, strokes, and polygon interiors; generated mixed-style file cross-opened; future-object default pending |
| Text | Partial: selected free-text captions and labeled points can toggle Bold, Italic, and Underline, step through verified size codes 14, 16, 18, 24, 36, 48, and 64, and choose a printable ASCII font family. The sidebar can select caption characters and toggle Bold, Italic, Underline, Larger, Smaller, or a font family for that range. The writer adds or reuses indexed `0x04b1` families and `0x04b3` styles and updates each selected object's `0x07d8` reference. Mixed-font captions keep their separate families through style edits and copy/paste. Generated Arial/CJK captions and an Arial point label opened and saved byte-identically in Sketchpad 5. Native and port-generated mixed-format captions render, move, copy, and edit; boundary insertion, replacement, and newline edits opened and saved byte-identically in Sketchpad 5. Native 36-point substring, port-generated 48-point substring, native mixed-family caption, and port-generated selected-font caption opened and saved byte-identically in Sketchpad 5. SVG exports include family and size per run; the desktop canvas loads installed fonts through fontconfig and falls back to its proportional font when a match cannot be loaded. Ambiguous repeated-character edits, other sizes, and broader rich-text commands pending |
| Hide Objects | Partial |
| Show All Hidden | Partial |
| Show Labels | Partial: point-label visibility through editor controls; other object labels pending |
| Label Objects | Partial: point labels can be named, styled, copied, and saved; other object labels pending |
| Trace Objects | Partial: point, segment, ray, line, circle, arc, and polygon trace flags, save/reopen, undo/redo, and bounded drag or animation trails; native ray, line, circle, arc, and polygon flags confirmed, polygon fill and full-line trails visually confirmed, and port-traced point, segment, ray, line, circle, arc, and polygon files opened and saved byte-identically in Sketchpad 5. Action-button traces and other object kinds pending |
| Erase Traces | Partial: clears native in-memory trails; original application's trail clearing verified for a port-generated traced point |
| Animate Objects | Partial: selected free points wander within the visible canvas, and points on segments, circles, and arcs animate and update dependents. A segment-driven polygon locus and a free point moved visibly in the desktop; their stopped files cross-opened byte-identically in Sketchpad 5. Native free-point path details and other motion modes pending |
| Increase Speed | Partial: doubles running constrained-point animation speed, bounded at 8× |
| Decrease Speed | Partial: halves running constrained-point animation speed, bounded at 0.125× |
| Stop Animation | Partial: stops the current constrained-point animation while keeping its current point positions |
| Hide/Show Motion Controller | Partial: floating pause, resume, stop, and speed controls for constrained-point animation |
| Hide/Show Toolbox | Pending |
| Hide/Show Text Palette | Pending |

## Construct

| Command | Status |
| --- | --- |
| Point on Object | Partial: segment, circle, and arc hosts, including drag and generated-file cross-open |
| Midpoint | Partial |
| Intersection | Partial: straight, line-circle, and circle-circle constructions; generated examples opened in Sketchpad 5 |
| Segment | Partial |
| Ray | Partial |
| Line | Partial |
| Parallel Line | Partial |
| Perpendicular Line | Partial |
| Circle by Center+Point | Partial |
| Angle Bisector | Partial: original fixture and generated ray cross-opened |
| Circle by Center+Radius | Partial: segment radius variant cross-opened |
| Arc on Circle | Partial: constrained endpoints and generated arc cross-opened |
| Arc through 3 Points | Partial: original fixture and generated arc cross-opened |
| Interior | Partial: polygon from selected vertices, fill and outline, native hit testing, SVG, generated file opened in Sketchpad 5 |
| Locus | Partial: segment-, circle-, or arc-constrained points can drive a dependent point, segment, ray, line, circle, arc, or polygon interior through supported constructions. Native code 35 point curves and segment, ray, line, circle, arc, and polygon families import as editable objects, render in the desktop and SVG views, and can be constructed from the selected pair. Native midpoint, translated-point, circle-driver, arc-driver, two-step, two-branch, segment-family, direct-ray-family, bisector-ray-family, direct-line-family, parallel-line-family, perpendicular-line-family, circle-family, three-point-arc-family, circle-arc-family, base-polygon-family, vector-translated-polygon-family, polar-translated-polygon-family, and rectangular-translated-polygon-family examples were reproduced; generated files opened and saved byte-identically in Sketchpad 5. Other traced object kinds and locus options pending |

## Transform

| Command | Status |
| --- | --- |
| Mark Center | Partial: marked point can be used for later rotate and dilate commands; original marked-only file is byte-identical to its unmarked source |
| Mark Mirror | Partial: marked straight object can be used for later reflection; original marked-only file is byte-identical to its unmarked source |
| Mark Angle | Partial: three selected points define a dynamic angle for rotation; a marked-only native save is byte-identical to its unmarked source |
| Mark Ratio | Partial: three collinear points define a signed dynamic ratio for point, segment, circle, and polygon dilation; a marked-only native save is byte-identical to its unmarked source |
| Mark Vector | Partial: the last two selected points define a dynamic vector for later translation; a marked-only native file is byte-identical to its unmarked source |
| Mark Distance | Partial: one selected distance measurement supplies a dynamic polar distance; two selected measurements supply horizontal and vertical distances in selection order; marked-only native saves are byte-identical to their unmarked sources |
| Translate | Partial: points and polygon interiors by fixed polar distance, fixed rectangular distances, marked polar distance, marked rectangular distances, either mixed fixed/marked rectangular mode, or marked vector; segments and circles by fixed polar, fixed rectangular, marked polar distance, or marked vector. Generated files for these modes opened and saved byte-identically in Sketchpad 5 after moving their dependencies; a generated 180° polar point translation also cross-opened after its angle cache was corrected. Other shapes and modes pending |
| Rotate | Partial: points, segments, circles, and polygon interiors around a selected or marked center by a fixed or three-point marked angle; generated shape and polygon files opened and saved byte-identically in Sketchpad 5 after moving a dependency; other shapes and angle sources pending |
| Dilate | Partial: points, segments, circles, and polygon interiors by fixed or three-point marked ratio around a center point; generated shape and polygon files opened and saved byte-identically in Sketchpad 5 after moving a dependency; other shapes and ratio sources pending |
| Reflect | Partial: points, segments, circles, and polygon interiors across a selected segment or line. A generated polygon file with a moved source vertex displayed both interiors in Sketchpad 5 and saved byte-identically; the native reflected polygon fixtures import without partial view. Other shapes pending |
| Iterate | Pending |
| Define Custom Transform | Pending |
| Edit Custom Transforms | Pending |

## Measure

| Command | Status |
| --- | --- |
| Length | Partial: segment length, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Distance | Partial: two-point distance, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Perimeter | Partial: base and transformed polygon interior perimeter, dynamic value, SVG, generated files edited in Sketchpad 5 |
| Circumference | Partial: circle circumference, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Angle | Partial: three-point angle, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Area | Partial: circle plus base and transformed polygon interior area, dynamic value, SVG, generated files opened and edited in Sketchpad 5; other area sources pending |
| Arc Angle | Partial: three-point arc edited; circle-based arc opened; dynamic value and SVG |
| Arc Length | Partial: three-point arc edited; circle-based arc opened; dynamic value and SVG |
| Radius | Partial: circle radius, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Ratio | Partial: signed point ratio, dynamic projection after drag, SVG, generated positive and negative files opened in Sketchpad 5 |
| Value of Point | Partial: segment/circle constrained points, live values, save/reopen, and generated files cross-opened |
| Coordinates | Partial: Cartesian point pair updates after drag, saves/reopens, generated file opened and saved in Sketchpad 5; complex names/formats pending |
| Abscissa (x) | Partial: Cartesian x value updates after drag, saves/reopens, generated file opened and saved in Sketchpad 5 |
| Ordinate (y) | Partial: Cartesian y value updates after drag, saves/reopens, generated file opened and saved in Sketchpad 5 |
| Coordinate Distance | Partial: Cartesian two-point value updates after drag, saves/reopens, generated file opened and saved in Sketchpad 5 |
| Slope | Partial: Cartesian frame and segment slope, live update, generated file edited in Sketchpad 5 |
| Equation | Partial: Cartesian line and circle equations, live values, save/reopen, and generated files cross-opened; other equation forms pending |

## Number

| Command | Status |
| --- | --- |
| Number Menu &gt; New Parameter | Partial: default t₁/t₂ parameters and multiple named ASCII parameters in a supported single-page sketch, including default parameters after a free point; editable −99.99 to 99.99 values at up to two-decimal precision; original and generated named parameters decode and cross-open; Unicode names and larger values pending |
| Number Menu &gt; Calculate | Partial: literal 2+3 expression, move and save/reopen; original file cross-opened |
| Number Menu &gt; Tabulate | Partial: one-column default t₁ table with cached values from −99.99 to 99.99 and a later source ID after a free point, named ASCII parameter tables (including a later source ID), and two-column t₂/t₁ table with up to 256 rows and values from −99.99 to 99.99 in both columns, including parameter IDs after a free point; original and generated mixed-width tables cross-opened; other sources and layouts pending |
| Number Menu &gt; Add Table Data | Partial: add default t₁ or named-parameter snapshot rows, or t₂/t₁ rows up to 256; fractional t₁ snapshots and named tables preserve earlier rows while appending the live value; generated t₁ and pair tables opened and saved in Sketchpad 5; other table kinds pending |
| Number Menu &gt; Remove Table Data | Partial: remove last default t₁, named-parameter, or t₂/t₁ row; earlier pair-table caches and plotted points stay fixed, and a generated four-row result opened and saved byte-identically in Sketchpad 5; other table kinds pending |
| Number Menu &gt; New Function | Partial: arithmetic expressions in `x` with integer or decimal constants, parentheses, unary signs, `+ − × ÷ ^`, and `sin`/`cos`/`tan` decode, evaluate, render, and save. Double-clicking a displayed formula edits its expression and updates dependent plots; edits that change token length preserve the saved function wrapper. New and edited trig expressions offer the radians/degrees choice; the document angle unit supports undo/redo and save/reopen. Named values, nested definitions, and further functions pending |
| Number Menu &gt; Define Derivative Function | Partial: a selected function definition or derivative produces a code-78 derivative with a symbolic expression. Native identity, square, cubic, sine, and degree-sine derivatives import and round-trip; generated square and sine derivatives in radians and degrees opened, displayed, and saved byte-identically in Sketchpad 5. Editing a source formula refreshes its derivative cache and dependent plots; an edited cubic source cross-opened. A degree-based trigonometric source offers the radians/degrees choice. Constant exponents and the supported arithmetic/trigonometric expression grammar work; variable exponents, other function types, and longer expressions pending |
| Number Menu &gt; Define Function from Drawing or Picture | Pending |

## Graph

| Command | Status |
| --- | --- |
| Graph Menu &gt; Define Coordinate System | Partial: default frame; selected origin, circle, one defining distance, or two defining distances with optional origin; segment, segment-length value, and two-point distance value work as defining distances. Circle and distance axis ticks and labels render in native/SVG. Native examples decode; generated square, rectangular, and one-distance Polar frames opened with their grids and saved byte-identically in Sketchpad 5. Broader axis and grid-form editing pending |
| Graph Menu &gt; Mark Coordinate System | Partial: selecting an axis, origin, unit point, or square/Polar grid intersection marks its frame for later plots and measurements; the page link survives undo, save, and reopen; an older-frame Cartesian plot opened and saved byte-identically in Sketchpad 5; native grid-click marking changed the same link |
| Graph Menu &gt; Grid Form | Partial: Square, Polar, Rectangular, and trigonometric X-axis formats decode and save; polar rings/rays, independent vertical spacing, and π labels render in native/SVG. One-distance frames convert among Square, Rectangular, and Polar while preserving whether the Y unit follows the defining distance (code 56) or has its own spacing (code 57); two-distance frames convert between Rectangular and elliptical Polar. Generated conversions cross-open byte-identically. Two-distance Square behavior and broader axis formatting pending |
| Graph Menu &gt; Show Grid | Partial: coordinate-system visibility decoded and saved; hidden frame components omitted from native/SVG rendering; original and generated hidden frames reopened in Sketchpad 5 |
| Graph Menu &gt; Dotted Grid | Partial: original grid flag decoded, native and SVG dots rendered, undo/redo and save/reopen tested; generated Cartesian and Polar dotted frames opened and saved byte-identically in Sketchpad 5 |
| Graph Menu &gt; Snap Points | Partial: page flag decoded and saved; free-point creation and drag snap to the nearest Cartesian or Polar grid intersection; visible grid remains drawn while snapping, matching the original; original drag results matched numerically, and port-generated file opened, snapped again, and saved in Sketchpad 5; other point kinds pending |
| Graph Menu &gt; Plot Value on Axis/Path | Partial: selected default or named parameter plots as a live code 95 point, while the value dialog plots a fixed numeric code 123 point; X/Y axes, segments, rays, lines, and circles decode, render, and save; original and generated cases opened and saved in Sketchpad 5; expressions, other numeric sources, and further paths pending |
| Graph Menu &gt; Plot Points/Plot as (x, y)/Plot as (r, θ) | Partial: supported one-page sketches with an active coordinate frame can plot entered Cartesian or polar coordinates, including multiple points. Original files with a parameter, free point, or text as the first object decode and round-trip; their generated point groups match. Generated mixed-layout and multi-point sketches opened and saved byte-identically in Sketchpad 5. Plotted points follow frame drags. Multi-page plotting and broader source layouts pending |
| Graph Menu &gt; Plot Table Data | Partial: one- to 256-row Cartesian or polar plot from the supported t₂/t₁ table, including values from −99.99 to 99.99 in both columns and a table and frame after a free point; duplicate rows combine, distinct rows produce separate points, and parameter edits update the live table row while plotted points stay fixed. A supported table can be plotted again after an edit. Original fractional and distinct-row polar files decode and round-trip; generated five-distinct-row, repeated-plot, shifted-ID polar, and mixed-width Cartesian files cross-opened byte-identically. Other column sources pending |
| Graph Menu &gt; Plot Function/Plot New Function | Partial: clicking a function formula selects its definition; Plot Function reuses that definition without adding a second one. Cartesian arithmetic expressions and `sin`/`cos`/`tan` in `x` plot and update after origin drag. A derivative can also be plotted; a generated square derivative plot opened and saved byte-identically in Sketchpad 5. Double-clicking a plot edits its numeric domain and sample count; original and newly created range edits cross-opened byte-identically. Tangent and reciprocal curves leave asymptote gaps in native/SVG rendering and hit testing. Original trig, compound, decimal, and reciprocal formulas decode; generated existing-definition, tangent, reciprocal, sine, combined trig, compound, and decimal plots opened and saved byte-identically in Sketchpad 5. Other sources, function types, arrowheads, and advanced discontinuities pending |
| Graph Menu &gt; Plot Parametric Curve | Partial: two selected or chosen functions, including derivatives, plot a rectangular or polar curve over an editable numeric domain. Native code-126/127 files import as editable documents; formula edits update their curves. Double-clicking a plot edits its numeric domain and sample count; original and newly created range edits cross-opened byte-identically. Generated rectangular, polar, custom-domain, and derivative-source files opened, rendered, and saved byte-identically in Sketchpad 5. The port renders curves in native UI and SVG. Function types beyond the current expression parser, arrowheads, and advanced discontinuities pending |

## Action Buttons

| Command | Status |
| --- | --- |
| Hide/Show | Partial: one- and two-target native buttons import as editable; clicking a two-point button hides or shows both points while leaving their segment visible. Port-generated two-target buttons and copied native two-target buttons opened and saved byte-identically in Sketchpad 5; clicking a generated button in Sketchpad 5 changed both points and the action state. Other target combinations and native copy placement pending |
| Animation | Partial: one- and two-point buttons import, create, save, and start or stop motion without the floating controller. Each target retains its own direction, speed, and Once Only settings. Native default, asymmetric second-direction, and clicked two-point files import and round-trip; the clicked button moved both points. Port-generated default and asymmetric two-point files opened and saved byte-identically in Sketchpad 5, and clicking the generated default moved both points. Desktop creation and per-point setting edits saved byte-identically to the replays. Free-point and circle defaults also match. More targets, indirect geometric targets, parameters, and exact timing pending |
| Movement | Partial: native one-pair point buttons import with Slow, Medium, Fast, Instant, Follow, and initial-destination settings and round-trip byte-identically. The port creates matching editable records, presents the settings in a dialog, and moves a free point toward a destination point. Port-generated default and two-button files opened and saved byte-identically in Sketchpad 5; clicking the generated default there moved the source onto the destination. A desktop click also moved the source, and that saved file cross-opened byte-identically. Parameter/value pairs, multiple pairs, and exact motion rates pending |
| Presentation | Partial: native simultaneous and sequential buttons import as editable and round-trip byte-identically. The port creates either mode for selected action buttons and runs Hide/Show, Animation, Movement, Scroll, and nested Presentation actions. Sequential mode waits for active motion before advancing; clicking again advances or stops a running child. Both port-generated modes opened and saved byte-identically in Sketchpad 5, and native clicks on each hid both selected points and changed both parent actions to Show. A desktop click gave the same saved state, which then opened and saved byte-identically in Sketchpad 5. Additional timing, presentation settings, and child combinations pending |
| Link | Partial: native default, custom, and even-length URL buttons import as editable and round-trip byte-identically. The port creates URL Link buttons, renders them, and opens HTTP(S), file, and sibling `sketchdoc://` links; a generated URL file cross-opened byte-identically, and a Rust click followed a sibling document link. Native Page Links to pages 1 and 2 import losslessly, and editor clicks switch to their saved destinations. The port creates Page and URL Links on any page of supported multi-page documents. Generated files from both the first and last pages opened and saved byte-identically in Sketchpad 5; native clicks navigated to their target pages. Native Page Links that activate a destination Hide/Show button import, create, and run in the Rust editor; a port-generated file opened and saved byte-identically in native Sketchpad, and clicking it there hid the destination point. The Rust editor can save the resulting Hide/Show state in a multi-page file; that file opened and saved byte-identically in native Sketchpad. General multi-page editing, other destination action types, and other special URL schemes pending |
| Scroll | Partial: native Upper Left and Center buttons import as editable and round-trip byte-identically. The port creates either mode, draws and activates the button, and pans action buttons with the viewport and SVG/PDF export. Generated one- and two-button files opened and saved byte-identically in Sketchpad 5. Clicking the port-generated Upper Left button there placed its point at the canvas corner; a desktop click did the same and moved the button out of view. Broader point/document layouts pending |

## Custom Tools

| Command | Status |
| --- | --- |
| Create New Tool | Pending |
| Use a Custom Tool | Pending |
| Document tools | Pending |
| Tool Folder tools | Pending |
| Choose Tool Folder | Pending |
| Forget Tool Folder | Pending |

## Tool and document workflows

| Workflow | Status |
| --- | --- |
| Select and drag free points | Partial: existing free points on supported multi-page files can be moved and saved. A Rust move matched native Sketchpad except for three opaque page-header bytes and cross-opened byte-identically. Moving a first-page point together with adding a Page Link also cross-opened byte-identically |
| Point tool | Partial |
| Segment tool | Partial |
| Circle tool | Partial |
| Text, image, table, iteration, plotting | Partial: literal text, point labels, a single PNG picture, and Cartesian `x`/`x²` plots cross-open. Literal captions with saved bold, italic, underline, and size records import as editable text; edits in mixed-style two-caption sketches cross-opened byte-identically. Bold, Italic, and Underline formatting of free text also cross-opened. Other text formatting, table, and iteration pending |
| Import, export, print | Partial: single PNG import, SVG/PDF export, and Linux printing for supported visible geometry; other image formats and physical print verification pending |
| Multi-page documents | Partial: page navigation and read-only complex objects |
| Cross-open generated files in Windows Sketchpad 5 | Partial: nine-object file edited and saved in Sketchpad 5, then edited in Rust and reopened under Wine |
