# Sketchpad 5 command parity

Source: the bundled Sketchpad 5 Reference Center menu pages. Status describes the current Rust port. **Partial** means a UI command exists but its full Sketchpad behavior and Windows cross-open have not yet been verified. **Pending** means absent.

The release gate is every sketch and document command verified after editing, saving, reopening, and cross-opening in Sketchpad 5. This gate is not met.

## File

| Command | Status |
| --- | --- |
| New Sketch | Partial |
| Open | Partial |
| Save | Partial: menu and Ctrl+S; an animation-stopped file saved from the desktop opened and resaved byte-identically in Sketchpad 5. The selected page persists even after navigation without content edits, including Page Links; first-page, second-page, and added-page saves reopen on the intended page and cross-open byte-identically |
| Save As | Partial: menu and Ctrl+Shift+S; broader document layouts pending |
| Close | Partial: unsaved-change dialog |
| Document Options | Partial: Add Blank Page, Rename Page, Remove Page, and Move Page work on supported sketches with Undo/Redo. Five generated reordered files with points, titles, a segment, and a remapped Page Link opened and saved byte-identically in native Sketchpad; double-clicking the moved link opened its new destination. Page removal can also preserve a Page Link to the deleted page as an unavailable button, matching native Sketchpad; the generated one- and two-page results opened and saved byte-identically there. Pictures and other options pending |
| Page Setup | Partial: A4 or Letter paper, portrait or landscape orientation, 0–40 mm margins, fit-to-page, and 25–400% custom scaling. Settings apply to PDF export, Print Preview, and Print without modifying the sketch; printer-specific paper trays and persisted preferences remain pending |
| Print Preview | Partial: displays the exact page-sized SVG that PDF export and Print consume, rasterized through `rsvg-convert`, with refresh and direct Print controls. Multi-page imposition and printer-specific previews remain pending |
| Print | Partial: visible page sent as PDF to the Linux default printer |
| Quit | Partial: unsaved-change dialog |
| Sketchpad LessonLink... | Outside release target (legacy service) |
| Sketchpad Sketch Exchange | Outside release target (legacy service) |

## Edit

| Command | Status |
| --- | --- |
| Undo | Partial |
| Redo | Partial |
| Cut | Partial: supported single-page objects and their dependents; segment Cut/Paste on a supported multi-page page opened and saved byte-identically in native Sketchpad |
| Copy | Partial: selected supported objects and required parents, including segment and midpoint cases observed in Sketchpad 5; one- and two-column tables include their source parameters and embedded source IDs; copied function plots retain their frame style and definition, and derivatives retain their source definition. Copying a selected function includes its Cartesian plotted child; copying either parametric source includes both functions and the curve, as observed in Sketchpad 5 |
| Paste | Partial: internal clipboard, 20-unit offset, and generated segment cross-opened in Sketchpad 5; copied one- and two-column tables remap their source IDs and pasted number/table anchors have the native 20-unit offset. Generated copied function plot, parametric curve, and derivative files opened and saved byte-identically in Sketchpad 5, with separate formula positions visible. A native copied plot imports as editable; its copied frame origin was moved in Rust and saved byte-identically in Sketchpad 5. Pasted plots hide their copied frame parts. Both choices of a selected parametric source receive the native `f`/`g` names. Segment and polygon-perimeter chains can also be copied and pasted on the same multi-page page or an earlier page; generated examples opened and saved byte-identically in native Sketchpad. Other graph copies and multi-page object families remain pending Pasting into proved legacy free-point/polygon and primitive-stroke documents upgrades all short headers while preserving metadata, fonts, point size, polygon opacity, and pen meaning, with structural undo; actual 2,072- and 2,696-byte writer outputs save byte-identically in native Sketchpad. See [legacy conversion evidence](dev/legacy_document_paste_analysis.md). Other legacy layouts remain pending |
| Clear | Partial: selected objects and dependent constructions in supported single-page files; generated result reopened in Sketchpad 5. Saved trailing function, Cartesian and parametric plot, and plotted-point groups opened and saved byte-identically in Sketchpad 5 after removal. Independent free points before surviving function plots, parametric plots, plotted points, and one- or two-column tables, plus an independent caption before a function plot, can be deleted with references remapped. Deleting a parameter removes its dependent table and path point while preserving an unrelated frame. Supported multi-page construction, measurement, and action-button groups can now be removed with page-local references remapped. Three generated action deletion files opened and saved byte-identically in native Sketchpad; a Presentation button whose references shifted still ran both parents there. Page Links, image resources, and other record families remain guarded |
| Action Buttons | Partial: Hide/Show for up to three verified targets, one- to seven-point Animation, multi-pair point and numeric Movement, one-point Scroll, Presentation of selected action buttons, URL Links, and Page Links on supported multi-page files; broader targets pending |
| Select All | Partial |
| Select Parents | Partial: direct dependencies of supported objects |
| Select Children | Partial: direct dependents of supported objects |
| Crop Picture to Polygon | Partial: one independent PNG and one live polygon in a supported single-page sketch; native code-108 records decode, render, save, and reopen, while attached and multi-resource picture layouts remain guarded |
| Split/Merge | Partial: Split Point detaches a supported midpoint, path point, intersection, coordinate/plotted point, or similar constructed point in a safe single-page sketch while preserving its object identity, style, label, and existing children. Merge Points reattaches the first-selected independent point's children to the second-selected point and removes the redundant source transactionally. Generated split and merge files with surviving child segments reopened in the port, then opened and saved byte-identically in native Sketchpad 5. Splitting transformed points, splitting an independent point into separate copies for multiple children, merging to paths, merging text or pictures, and multi-page files remain pending; see [Split/Merge evidence](dev/split_merge_analysis.md) |
| Edit Definition | Partial: the Edit menu changes to Edit Calculation, Edit Function, or Edit Parameter for one supported selection and opens the existing undoable editor. Source-free calculations, direct/referenced functions, and numeric parameter values save and reopen; converting a parameter to a calculation and editing other calculation families remain pending |
| Properties | Partial: one selected object opens a common Object panel with its type, saved label, hidden state, and direct parent/child links. Visibility changes are undoable, and related-object links update the selection and panel. Points, straight objects, circles, the three base arc constructions, transformed segments/rays/lines/circles/arcs, and live colorized straight/circle/arc outputs expose native label name and visibility editing with parameter-based placement in the canvas and SVG; Native-created controls import as full views, and actual port writers displayed these labels and saved byte-identically in Sketchpad 5. Polygon-style interiors expose their full 16-bit opacity through a percentage slider with transactional undo/save validation; an actual 75%-opacity polygon opened and saved byte-identically in Sketchpad 5. Labels on other geometry, picture/sampled opacity, Arrow Selectable, and specialized Value, Parameter, Table, Iteration, Axis, Function, Plot, Marker, and action panels remain pending. See [Properties evidence](dev/properties_analysis.md) |
| Preferences | Partial: the Units panel provides independent 0–5 decimal defaults for newly created angle, distance/area, and other measurements and supported calculations. The Tools panel supports Low, Medium, or High selection magnetism, Sketchpad's optional double-click deselection behavior, and the full native opacity range for newly created polygons; circle and sector interiors remain independent. The Color panel controls the canvas background, selection markers, six native future-object color classes (points, straights, circles/arcs, interiors, point loci, and plots), optional varied colors for new interiors, and time-based trace fading with an adjustable lifetime; the Text panel controls whether creating a caption opens the Text Palette. Future-object defaults reach ordinary and compound construction APIs without changing existing/imported objects, and authored values persist in saved files. When enabled, applying Display > Color updates every represented future-object class; holding Shift bypasses that update as in Native. Settings apply immediately and survive New/Open within the editor session. Angle and distance unit choices, automatic polygon-perimeter framing, remaining Text/Tools settings, exact Native trace-speed calibration, per-sketch persistence, and cross-session new-sketch defaults remain pending |

## Display

| Command | Status |
| --- | --- |
| Point Style | Partial: selected points, undo/redo, SVG/native rendering, save/reopen and generated file cross-open. Legacy Small point style imports correctly; changing it to Medium upgrades the document with structural undo and native byte-identical saving. Applied styles become the session default for future points, while Shift preserves the prior default |
| Line Style | Partial: width and solid/dashed/dotted patterns for supported strokes, including point and parameter loci, function plots, and rectangular/polar parametric plots. Generated segment/circle and five edited curve layouts opened and saved byte-identically in Sketchpad 5; SVG/canvas retain curve dash continuity. Legacy segment, circle, ray, and perpendicular strokes retain native Thin Dashed, Thin Solid, and Medium Solid meanings. A dotted-ray edit upgrades their short headers, with undo and native byte-identical saving; see [legacy style evidence](dev/legacy_document_paste_analysis.md). Width and pattern changes become independent future defaults for straights, rounds, point loci, and plots; Shift preserves prior defaults. Swept-locus styles remain pending; see [curve evidence](dev/curve_styles_analysis.md) |
| Color | Partial: RGB for supported points, strokes, polygon interiors, and the five supported point-curve families. Generated mixed-style file and five edited curve layouts cross-opened. Applied colors can update all six future-object classes with a Shift override; swept-locus formatting remains pending; see [curve evidence](dev/curve_styles_analysis.md) |
| Text | Partial: selected free-text captions, labeled points, and supported numeric measurement captions can toggle Bold, Italic, and Underline, step through verified size codes 14, 16, 18, 24, 36, 48, and 64, and choose a printable ASCII font family. The sidebar can select caption characters and toggle Bold, Italic, Underline, Larger, Smaller, or a font family for that range. The writer adds or reuses indexed `0x04b1` families and `0x04b3` styles and updates each selected object's `0x07d8` reference. Mixed-font captions keep their separate families through style edits and copy/paste. Native numeric captions retain their indexed fonts and saved positions, can be dragged on supported single-page and known-object multi-page sketches, and preserve their source placement plus the native `(20,20)` clipboard offset. Canvas picking and SVG use those records; see [numeric caption evidence](dev/numeric_caption_analysis.md). Supported multi-page sketches can create, edit, move, format, copy/paste, and clear literal captions, including mixed-format ranges; see [multi-page regression checks](dev/multi_page_text_analysis.md). Generated Arial/CJK captions and an Arial point label opened and saved byte-identically in Sketchpad 5. Native and port-generated mixed-format captions render, move, copy, and edit; boundary insertion, replacement, and newline edits opened and saved byte-identically in Sketchpad 5. Native 36-point substring, port-generated 48-point substring, native mixed-family caption, and port-generated selected-font caption opened and saved byte-identically in Sketchpad 5. SVG exports include family and size per run; the desktop canvas loads installed fonts through fontconfig and falls back to its proportional font when a match cannot be loaded. Ambiguous repeated-character edits, other sizes, and broader rich-text commands pending |
| Hide Objects | Partial |
| Show All Hidden | Partial |
| Show Labels | Partial: the Display menu and selected-object controls show or hide existing point, segment, ray, line, constructed-straight, circle, base-arc, transformed straight/circle/arc, and colorized straight/circle/arc labels in one undoable transaction; other object labels pending |
| Label Objects | Partial: point, segment, ray, line, constructed-straight, circle, base-arc, transformed segment/ray/line/circle/arc, and colorized straight/circle/arc labels can be named and saved; point labels can also be styled and copied. Native controls for these geometry families import as full views. Port-authored subscripted labels for every listed family displayed and saved byte-identically in Sketchpad 5. Other object labels pending |
| Trace Objects | Partial: point, segment, ray, line, circle, arc, and polygon trace flags, save/reopen, undo/redo, and bounded drag or animation trails; native ray, line, circle, arc, and polygon flags confirmed, polygon fill and full-line trails visually confirmed, and port-traced point, segment, ray, line, circle, arc, and polygon files opened and saved byte-identically in Sketchpad 5. Action-button traces and other object kinds pending |
| Erase Traces | Partial: clears native in-memory trails; original application's trail clearing verified for a port-generated traced point |
| Animate Objects | Partial: selected parameters animate over their stored domain and update dependent tables; free and transformed points wander within the visible canvas; and points on segments, rays, lines, circles, arcs, polygon boundaries, supported point curves, and sector boundaries animate and update dependents. Transformed-point motion is a temporary evaluator override: dependents follow it, Stop restores the construction-defined position, and Save retains the original construction. Circle, polygon, and sector paths loop; bounded open paths reverse at their endpoints. Selecting a segment or circle animates its source points, including hidden endpoints; native Sketchpad 5 saved changed positions for both sources. A segment-driven polygon locus and a free point moved visibly in the desktop; their stopped files cross-opened byte-identically in Sketchpad 5. Display parameter step/domain controls, native free-point path details, precise timing, and other motion modes pending |
| Increase Speed | Partial: doubles running constrained-point animation speed, bounded at 8× |
| Decrease Speed | Partial: halves running constrained-point animation speed, bounded at 0.125× |
| Stop Animation | Partial: stops current motion; directly editable points keep their current positions, while temporary transformed-point overrides return to their construction-defined positions |
| Hide/Show Motion Controller | Partial: floating pause, resume, stop, and speed controls for constrained-point animation |
| Hide/Show Toolbox | Partial: Display toggles the tool buttons and text-placement controls; page navigation and selected-object controls remain available. Native floating layout and saved preferences pending; see [desktop checks](dev/desktop_controls_analysis.md) |
| Hide/Show Text Palette | Partial: Display toggles a floating palette for selected text, point labels, and numeric captions. The palette applies Bold, Italic, Underline, Larger, Smaller, and a font family through the same undoable document commands as Display > Text. Native palette layout and preference persistence remain pending |

## Construct

| Command | Status |
| --- | --- |
| Point on Object | Partial: segment, ray, line, circle, arc, and polygon boundary hosts, including drag and generated-file cross-open; creation works on supported multi-page pages. Native ray/line constraints import and their generated object groups match byte for byte. Polygon boundary points retain the native edge index and local fraction, follow vertex changes, support linked perimeter values and closed-path animation, and save on supported multi-page pages. Known transformed polygon hosts also support creation, live vertex edits, dragging, and dependency copying; actual generated/copied files saved byte-identically in Sketchpad. A generated dragged point and its linked value displayed correctly and saved byte-identically in Sketchpad 5 Point-target geometric and parameter loci now support a shared constrained-point definition, projection, dragging, dependency edits, undo and page-local saving; 60 Native controls prove analytic evaluation and source-specific bounds. Function-plot free points now use normalized stored parameters, canonical function-x readouts and bounded nearest-curve dragging; all13 public API static controls match independently saved Native files, with two Native interactive drag controls. Swept targets remain pending. See [function point evidence](dev/function_points60_native_analysis.md) and [curve point evidence](dev/path_curve_points58_analysis.md). |
| Midpoint | Partial: supported multi-page segment hosts; generated object group matches native Sketchpad byte for byte and cross-opens |
| Intersection | Partial: straight, line-circle, circle-circle, straight-arc, circle-arc, and arc-arc constructions are live and save on supported pages. Native-created controls confirm code 11/12 and 13/14 creation order; round/arc controls prove branch identity for both operand orders. A clipped one-hit control creates only code 13, while either encoded branch retains Native's sole-hit geometry. The editor suppresses undefined or duplicate tangent branches. Port-authored circle-arc and arc-arc files display both crossings and save byte-identically in Sketchpad 5. A 20-object two-page writer containing both branches of straight-arc, circle-arc, and arc-arc intersections and a separate page-2 polygon-boundary writer also saved byte-identically in Native. Supported sampled-curve intersections are available; broader host families and sampled-curve multi-page layouts remain pending. See [round/arc evidence](dev/ANALYSIS.md#round-arc-intersections) and [polygon-boundary evidence](dev/ANALYSIS.md#polygon-boundary-intersections). |
| Segment | Partial |
| Ray | Partial: supported multi-page point pairs; generated object group matches native Sketchpad byte for byte and cross-opens |
| Line | Partial: supported multi-page point pairs; generated object group matches native Sketchpad byte for byte and cross-opens |
| Parallel Line | Partial: supported multi-page point and straight-path parents; generated group matches a native construction byte for byte and cross-opens |
| Perpendicular Line | Partial: supported multi-page point and straight-path parents; generated group matches a native construction byte for byte and cross-opens |
| Circle by Center+Point | Partial |
| Angle Bisector | Partial: works on supported multi-page pages; generated page 2 file opened and saved byte-identically in native Sketchpad |
| Circle by Center+Radius | Partial: segment radius variant on supported multi-page pages; generated object group matches native Sketchpad byte for byte and cross-opens |
| Arc on Circle | Partial: general typed point endpoints use the circle center, start-point radius and end-point direction. Degenerate centers/angles retain undefined dependents. Fourteen public API controls match independently saved Native files entirely; page-local edits and undo pass. See [arc evidence](dev/arc_sources59_native_analysis.md) |
| Arc by Center | Partial: construction, measurements, constrained/value/literal points and multi-page serialization; defined and degenerate API outputs opened and saved byte-identically in native Sketchpad |
| Arc through 3 Points | Partial: works on supported multi-page pages; generated page 2 file opened and saved byte-identically in native Sketchpad |
| Interior | Partial: modern Circle interiors retain an independent live Circle source, style, visibility and fill order; public API8 files match independent Native saves, with shared rendering/picking, undo and page-local writing. Legacy Circle-interior opacity and disk transforms remain pending. See [Circle evidence](dev/circle_interior60_native_analysis.md). Analytic sector interiors support closed-perimeter points, area, perimeter and arc-only curve length, shared fills/picking, drag/undo and page-local writing; native name/style metadata is preserved as inert presentation data for 28 additional drawing/tool-local sectors. See [sector evidence](dev/sector_boundary59_native_analysis.md). Generic sector transformations and aliases remain pending. Polygon from selected vertices, fill and outline, native hit testing, SVG, and signed native drawing order with stable ties; imported/copy/pasted ranks persist. Works on supported multi-page pages, including an interior on each page in one file. Generated multi-page files and coincident rank-swapped interiors opened and saved byte-identically in Sketchpad 5; legacy GSP4 interiors retain their short headers through point edits |
| Locus | Partial: segment-, circle-, or arc-constrained points can drive a dependent point, segment, ray, line, circle, arc, or polygon interior through supported constructions. Native code 35 point curves and segment, ray, line, circle, arc, and polygon families import as editable objects, render in the desktop and SVG views, and can be constructed from the selected pair. Native midpoint, translated-point, circle-driver, arc-driver, two-step, two-branch, segment-family, direct-ray-family, bisector-ray-family, direct-line-family, parallel-line-family, perpendicular-line-family, circle-family, three-point-arc-family, circle-arc-family, base-polygon-family, vector-translated-polygon-family, polar-translated-polygon-family, and rectangular-translated-polygon-family examples were reproduced; generated files opened and saved byte-identically in Sketchpad 5. A scalar parameter can drive a dependent Cartesian numeric point as a native code-97 trace; source edits, origin movement, creation, clipboard copying, and range edits cross-opened byte-identically. Existing PointOnCurve, Polygon and Sector constrained drivers also replay their analytic/perimeter path through the shared evaluator;12 Native controls and12 public API files are whole-file exact, including edited/closed/coarse/dense cases. Intrinsic sampled102 point traces preserve explicit carried/frozen boundaries and accept proved Segment, Circle and Arc profiles. Literal15/90 and Scalar95/literal123 children retain separately verified parameter bounds. See [sampled evidence](dev/sampled_mapped61_analysis.md) and [driver evidence](dev/locus_drivers61_analysis.md). Other traced object kinds and locus options pending |

## Transform

| Command | Status |
| --- | --- |
| Mark Center | Partial: marked point can be used for later rotate and dilate commands; original marked-only file is byte-identical to its unmarked source |
| Mark Mirror | Partial: marked straight object can be used for later reflection; original marked-only file is byte-identical to its unmarked source |
| Mark Angle | Partial: three selected points or one numeric value define a dynamic angle for rotation. Numeric magnitudes are interpreted in degrees regardless of source units; see [native quantity controls](dev/numeric_quantity_operands.md). A marked-only native save is byte-identical to its unmarked source |
| Mark Ratio | Partial: three collinear points or a selected numeric value define a signed dynamic dilation ratio. Numeric magnitudes are used directly regardless of source units; see [native quantity controls](dev/numeric_quantity_operands.md). A marked-only native save is byte-identical to its unmarked source |
| Mark Vector | Partial: the last two selected points define a dynamic vector for later translation; a marked-only native file is byte-identical to its unmarked source |
| Mark Distance | Partial: one selected distance measurement supplies a dynamic polar distance; two selected measurements supply horizontal and vertical distances in selection order; marked-only native saves are byte-identical to their unmarked sources |
| Translate | Partial: points and polygon interiors by fixed polar distance, fixed rectangular distances, marked polar distance, marked rectangular distances, either mixed fixed/marked rectangular mode, or marked vector; segments and circles by all those translation modes. Fixed rectangular axes retain native centimeter or inch units; ten original inch-unit Segment translations import live, and a port-authored inch translation resaved byte-identically in Sketchpad 5. Marked rectangular and both mixed fixed/marked segment/circle modes support live dependencies, clipboard copying, and save/reopen; a generated six-mode file opened and saved byte-identically in Sketchpad 5 after moving both defining distances. Generated files for these modes opened and saved byte-identically in Sketchpad 5 after moving their dependencies; a generated 180° polar point translation also cross-opened after its angle cache was corrected. Native vector-translated point names are retained, and vector translations can use supported transformed segments and circles as their sources. Four generated rotated-source files opened and saved byte-identically in Sketchpad 5; see [vector translation evidence](dev/vector_transform_chains_analysis.md). Fixed-polar and marked-distance translations also accept supported transformed segments, circles and polygons, with live two-step matrices, downstream measurements and save/undo/redo regression coverage. Observed native radian literal angles import with checked direction caches; fixed-polar import/writing now treats the native distance cache as total pixels, including fractional and negative magnitudes. A generated fractional-distance segment/circle/polygon chain opened and saved byte-identically in Sketchpad 5. Marked distance sources also include typed Length Parameters and calculation values via the normal Mark Distance command; live polar/Cartesian/mixed regressions and an actual generated parameter-driven file have Native byte-identical save evidence. Other shapes and modes pending. Analytic arcs now use the same transform carrier, live dependency evaluation and editor commands; all fourteen flavors have fresh Native cross-open controls, including signed and zero-scale chains. See [arc evidence](dev/transformed_arcs48_native_analysis.md). Ray/line sources, including supported derived and transformed paths, now support fixed-polar and marked-vector translation through the shared matrix model. Other straight translation modes remain guarded. See [straight source evidence](dev/straight_transform_sources54_analysis.md). |
| Circular-segment transforms | Partial: the observed code-19 mixed rectangular form, code-27 fixed rotation, code-30 fixed dilation, and code-34 reflection import and can be authored. Fill, picking, area, perimeter, and writing preserve arc-plus-chord semantics. Port-authored translation and combined fixed-transform controls rendered at the expected positions in Sketchpad 5. Marked-angle, marked-ratio, and other translation flavors remain guarded. See [evidence](dev/circular_segment_translation_analysis.md). |
| Rotate | Partial: points, segments, circles, and polygon interiors around a selected or marked center by a fixed or three-point marked angle; generated shape and polygon files opened and saved byte-identically in Sketchpad 5 after moving a dependency; A degree-valued calculation, measured angle, or arc-angle measurement can rotate a point, segment, circle, or polygon interior around a selected or marked center and updates with its numeric source; native shape records match the port byte for byte, and eight generated original/edited shape and calculated-length files cross-open and save byte-identically; generated regular polygon iterations cross-opened byte-identically. Legacy code-119 angle-value rotations import as the same live model for strict point, segment, and analytic-arc groups; seven roots and three dependents recover, while unsupported source families remain guarded. Calculated rotations can also use supported transformed segment and circle sources; compound dilation/rotation files and their negative-ratio edits save byte-identically in native Sketchpad. See [calculated transform chains](dev/calculated_transform_chains_analysis.md). Fixed-angle rotations also accept supported transformed segments and circles; see [native chain evidence](dev/fixed_transform_chains_analysis.md). Fixed rotations of primitive rays and lines preserve their full extent and support point reflection using their supporting lines; native original/edited/generated/mirror files save byte-identically. See [ray/line rotation evidence](dev/fixed_rotation_straights_analysis.md). Fixed rotation/dilation chains accept the four fixed transformed ray/line kinds, inherit source strokes at creation, and retain undefined zero-scale ancestors; eight authored and moved files save byte-identically in native Sketchpad. See [straight-chain evidence](dev/fixed_straight_transform_chains_analysis.md). The ordinary GraphAxis x-axis also supports the observed fixed and numeric-angle rotation schemas as an infinite line; a port-authored pair rendered correctly in Sketchpad 5, while other axis operations remain guarded. See [axis rotation evidence](dev/axis_rotation_analysis.md). Other shapes and angle sources pending. Analytic arcs now use the same transform carrier, live dependency evaluation and editor commands; all fourteen flavors have fresh Native cross-open controls, including signed and zero-scale chains. See [arc evidence](dev/transformed_arcs48_native_analysis.md). |
| Dilate | Partial: points, segments, circles, and polygon interiors by fixed, three-point marked, or scalar numeric ratio around a center point; generated shape and polygon files opened and saved byte-identically in Sketchpad 5 after moving a dependency; four native numeric-ratio object groups match the port byte for byte, and thirteen originals/edits verify parameters, calculations, measured ratios, negative/zero ratios, carried circles, and numeric iteration labels; Scalar-calculated dilations can use supported transformed segments and circles and feed a further calculated rotation. Four generated files save byte-identically in native Sketchpad; see [calculated transform chains](dev/calculated_transform_chains_analysis.md). Fixed-ratio dilation also accepts supported transformed segments and circles; see [native chain evidence](dev/fixed_transform_chains_analysis.md). Fixed dilation also supports primitive rays and lines, with negative ratios reversing ray direction and imported zero ratios retaining undefined geometry; fresh/edited/mirror-host files save byte-identically in native Sketchpad. See [ray/line dilation evidence](dev/fixed_dilation_straights_analysis.md). Other shapes and ratio sources pending. Analytic arcs now use the same transform carrier, live dependency evaluation and editor commands; all fourteen flavors have fresh Native cross-open controls, including signed and zero-scale chains. See [arc evidence](dev/transformed_arcs48_native_analysis.md). |
| Reflect | Partial: points, segments, circles, and polygon interiors across a selected segment or line. A generated polygon file with a moved source vertex displayed both interiors in Sketchpad 5 and saved byte-identically; the native reflected polygon fixtures import without partial view. Supported transformed segments and circles can also be reflected; native source/center/mirror edits and copied chains retain live geometry and save byte-identically. See [fixed transform chain evidence](dev/fixed_transform_chains_analysis.md). Other shapes pending. Analytic arcs now use the same transform carrier, live dependency evaluation and editor commands; all fourteen flavors have fresh Native cross-open controls, including signed and zero-scale chains. See [arc evidence](dev/transformed_arcs48_native_analysis.md). Ray/line sources, including Parallel, Perpendicular, Bisector and supported transform chains, now retain their extent through shared reflection geometry. See [straight source evidence](dev/straight_transform_sources54_analysis.md). |
| Iterate | Partial: one free point mapped to its directly translated, rotated, dilated, or reflected image at fixed depth 1–128 or with a scalar parameter/calculation for point-only depth, optionally carrying the connecting segment or ray. The ray form uses a scalar parameter or calculation for depth, and its points and rays update when that value or the seed changes. Native point, segment, and parameter-driven ray iterations import without partial view. Generated depth-3/depth-5 point, carried-segment, and parameter-driven ray files opened and saved byte-identically in Sketchpad 5; a port edit of the native ray fixture from depth 4 to 5 also cross-opened and saved byte-identically. Generated point and carried-segment rotation iterations also cross-opened byte-identically before and after moving the independent free center; the port updates the full orbit after seed or center edits. Combined point/parameter mappings with a direct parameter calculation and attached numeric label also import and can be created at fixed depth 1–128 or controlled by a distinct scalar depth parameter; their repeated values update with the parameter and their positions follow the point mapping. Generated and edited numeric iterations cross-opened and saved byte-identically in Sketchpad 5. Generated depth-4 numeric iterations, edits to depth 6 with a changed numeric seed, and a native depth-5 numeric fixture edited to depth 3 also cross-opened and saved byte-identically. Repeated numeric labels can select the iteration and their hit targets follow zoom, pan, and depth edits. Two directly translated images can branch their triangular interior and selected sides at fixed depth 1–128 or with a scalar depth parameter. The native 13-object example imports completely; edited and fixed-depth generated triangles cross-opened and saved byte-identically in Sketchpad 5. Desktop/SVG fills preserve coincident path multiplicity and saved opacity, and carried geometry can select its binding. A calculated-angle point can carry its connecting segment at a depth supplied by a scalar calculation. The native 10-object regular polygon example imports fully: `360°/n` supplies the rotation and `n-1` supplies the depth. Generated pentagon and edited hexagon files cross-opened and saved byte-identically in Sketchpad 5. Point-only calculated-angle rotation also supports a scalar calculation as depth, including `360°/n` with `n-1`; changes update the orbit and repeated point count together. Generated translation/rotation point iterations and their edited depth/source files opened and saved byte-identically in Sketchpad 5. Carried rays also support fixed-angle and calculated-angle rotation around a supported point center. Generated fixed-angle and `360°/n` ray iterations, plus edits to depth, angle source, seed, and center, opened and saved byte-identically in Sketchpad 5. Copy/paste preserves both the point and ray bindings. A circle centered on the free seed through its mapped image can also be carried through translation, fixed-angle rotation, or calculated-angle rotation, with fixed or scalar-calculated depth. Generated translated circles and calculated circle orbits at n=5 and n=6 rendered correctly and saved byte-identically in Sketchpad 5. Repeated circle bindings preserve stroke/color and visibility, support picking, and survive copy/paste. Numeric mappings accept two to sixteen formula inputs with the mapped parameter at any source index. External measurements/calculations stay fixed during each sequence; changing them refreshes all repeated labels. Five native-verified files cover secondary-source mapping, external parameter edits, measured-length inputs, live depth, and an external formula dependent on the seed. Fixed and marked-ratio point dilations now support point, segment, ray, circle, and combined numeric iterations. Centers and ratio inputs can be supported constructed points; edits update every step, radii scale, and negative factors alternate sides. Generated originals/edits open and save byte-identically in native Sketchpad. Reflected images and vector-translated images also support point/carried/numeric iteration through the shared evaluator. Reflected geometry alternates around the live mirror. Repeated direct seed references in a vector are substituted together; external dependencies stay pinned. Native probes cover stationary and doubling vectors, dependent external inputs, source/mirror edits, reflected circles, expanding vector circles, and collapsed circles. Undefined circles are omitted from drawing. Marked polar distances, marked and mixed rectangular translations, and three-point marked-angle rotations now use the same orbit evaluator. Every direct seed reference is substituted per step, including angle and ratio operands; external construction inputs remain pinned. Twelve native-verified original/edited files cover marked measurements, length calculations with numeric labels, both mixed translations, changing angles/ratios, and midpoint-defined centers. Direct measured-angle and arc-angle values can also drive rotation and iteration without an intermediate calculation; four original/edited segment and circle files display the expected live orbits and save byte-identically in native Sketchpad. Scalar numeric-ratio dilations also drive point, carried-shape, and numeric iteration. Parameters, formulas, and measured point ratios remain pinned within each sequence; negative and zero factors use the shared orbit evaluator. Native probes verify shrinking circles, edited fractional depth, measured-ratio dependencies, and expanding numeric labels. Multi-seed simultaneous free-point mappings through supported vector constructions now import, evaluate, save, copy, and clear, with carried segment bindings and fixed or live depth. Fresh one-branch, two-branch, live-depth, and edited-live examples cross-opened byte-identically; [native evidence and bounds](dev/mapped_point_iteration_analysis.md) document this family. Simultaneous midpoint mappings can carry selected point images as well as segments. Native medial-triangle iterations and generated/edited files render their nested triangles and point images, retain styles, support picking and clipboard operations, and save byte-identically; see [midpoint iteration evidence](dev/mapped_midpoint_iteration_analysis.md). Simultaneous mappings also accept calculated-angle rotations whose angle depends only on parameters and constants; native star iterations, source edits, and generated files retain carried points and segments and save byte-identically. Supported geometry-dependent angle formulas now use the shared carried-node evaluator; see [calculated-angle iteration evidence](dev/mapped_calculated_angle_iteration_analysis.md). Supported simultaneous mappings retain the native choice to show all levels or only the final level; selecting an iteration or its carried binding exposes that choice in the sidebar, with undo/redo and save/reopen; see [image-level evidence](dev/mapped_iteration_image_levels_analysis.md). Simultaneous mappings now accept derived point and Scalar seeds, including mixed roles, with typed branch targets and reevaluation of supported coordinate and transform DAGs. Generic point/segment transforms share the existing matrix and numeric evaluation; carried rotated/scaled segments, Fibonacci coordinate points, live depth edits and source restoration have exact actual API Native-save proofs. The original five-branch golden-ratio pentagon also renders all 25 final images; structural restrictions on partial imports remain. See [derived point and transform evidence](dev/mapped_derived_transform_analysis43.md) and [scalar/mixed state evidence](dev/numeric_mapped43_analysis.md). Ordinary geometric and measurement prerequisites now share that evaluator, including path points, line/line, line/circle and oriented circle/circle intersections, center arcs, angle and length measurements. Only explicitly carried nodes and seeds update within a sequence; external intermediates and function coefficient environments stay frozen. Undefined carried images preserve valid prefixes and other bindings. Native files prove full and abbreviated carried graphs and labeled API writers; see [path/intersection evidence](dev/mapped_path_intersection_analysis44.md) and [measurement/arc evidence](dev/measurement_mapped44_analysis.md). General mapped evaluation now accepts one to sixteen point/Scalar seeds, including single derived/path and calculated Scalar seeds. Original triangle and rectangular grids and eleven fresh API controls have exact Native proofs. Existing simple layouts retain canonical wire adapters; carried authoring orders actual construction predecessors first, including midpoint segment hosts. Binding styles, final-level edits, source restoration and undo/redo remain consistent. Original unsorted carried packets retain their bytes; their Native historical update-order artifact is documented. See [single-seed evidence](dev/single_seed_iteration_analysis45.md). Generic bindings now also support circles, polygons, rays, lines and arcs through shared output readers, SVG/GPU rendering and picking. The Iterate command accepts a selected point mapping, derived seeds, supported geometry and optional live Scalar depth. Binding styles remain independent; undefined outputs preserve other bindings and valid levels. Typed-Length circle radii and page-local numeric-group validation support original spiral controls. Actual API Native saves cover these output families, source edits, transformed/frozen hosts and undefined middle outputs; see [generic binding evidence](dev/iteration_bindings47_analysis.md) and [Native controls](dev/iteration_bindings47_native_analysis.md). Parameter-driven point-trace collections now share this scoped collection path: ten original mapped roots and 25 bindings decode, and a hidden-source Native control isolates the bound spiral. Typed numeric seeds beyond Scalar, broader mapping dialogs, undefined branch-target handling and further branch forms remain pending. |
| Define Custom Transform | Partial: selecting a pre-image point followed by a dependent image point defines a named point transform from the complete supported dependency slice. Applying the menu item to one point creates a live native code-101 mapped point; a generated midpoint-based application with a surviving child segment reopened in the port and saved byte-identically in Sketchpad 5. Definitions are document-session metadata and currently support point outputs only; native persistence, shapes, multiple inputs, and broader dependency families remain pending. See [custom-transform evidence](dev/custom_transform_analysis.md) |
| Edit Custom Transforms | Partial: the modal can rename, reorder, and remove current document-session definitions, and identifies definitions invalidated by construction edits. Definitions are cleared on New/Open and after a save that renumbers objects; native definition persistence and cross-document transfer remain pending |

Mapped iteration note: hide/show, animation, and movement actions can participate
as capture-only dependency nodes, but remain unavailable as repeated bindings.
This restores 21 mapped roots and 147 ordinary geometry bindings; a bindings-only
Native control displays the same red point and green arc families as the port.
Code-92 iteration tables retain every carried numeric/function column and render
the initial value plus each actual mapped generation at their native anchor.

## Measure

Length, distance, perimeter, circumference, angle, circle and polygon area, arc angle, arc length, radius, and signed point ratio can now be added on supported multi-page pages. Generated files containing all of these measurements opened and saved byte-identically in native Sketchpad 5. These twelve standard measurement kinds preserve saved decimal precision from 0–5 and support Measure → Decimal Places, including multiple selections, undo/redo, clipboard remapping, and supported multiple-page edits. Desktop captions and SVG use the saved precision while calculation inputs remain unrounded. Generated originals/edits cross-open in native Sketchpad with byte-identical saves. Scalar x/y coordinates, coordinate distance, slope, and Value of Point also support saved decimal precision 0–5, with full values retained for formulas and live iteration depth. Four original/edited scalar-precision files display the expected native results and save byte-identically. Measure → Decimal Places also edits all four supported calculation layouts on single-page sketches, with mixed measurement/calculation selections, undo/redo, clipboard preservation, attached numeric labels, and desktop/SVG captions. Native original/edited precision files preserve full input values and save byte-identically. Coordinate-pair/equation formatting and broader source variants remain pending.

| Command | Status |
| --- | --- |
| Length | Partial: original and transformed segment hosts, including numeric-angle rotations; segment length, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Distance | Partial: two-point and point-to-segment/ray/line supporting-line distance, including transformed straight hosts; live values, precision 0–5, calculations, marked-distance iteration, dependency remapping, and supported multi-page saving. Seven generated originals/edits open and save byte-identically in native Sketchpad 5. See `dev/point_line_distance_analysis.md` for native evidence |
| Perimeter | Partial: base and transformed polygon interiors plus sectors and circular segments, including mixed-translated circular segments, with analytic live values, SVG, and single- or multi-page authoring; generated polygon files edited in Sketchpad 5 |
| Circumference | Partial: base and transformed circle circumference, dynamic value, SVG; generated transformed-circle files opened and saved byte-identically in Sketchpad 5 |
| Angle | Partial: three-point angle, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Area | Partial: base and transformed circles and polygon interiors plus sectors and circular segments, including mixed-translated circular segments, with analytic live values, SVG, and single- or multi-page authoring; generated transformed-circle files opened and saved byte-identically in Sketchpad 5; other area sources pending |
| Arc Angle | Partial: three-point and center arcs edited; circle-based arc opened; dynamic value and SVG |
| Arc Length | Partial: three-point and center arcs edited; circle-based arcs opened; sectors and native circle-plus-two-endpoint measurements evaluate live. The editor and writers support the three-reference form on single- and multi-page sketches; 12 imported roots restore 61 descendants. Dynamic value and SVG are supported. |
| Radius | Partial: base and transformed circles plus supported circular arcs and sectors, with live values, calculations, SVG, save/reopen, and single- or multi-page authoring. Native CenterArc/SectorBoundary groups and generated transformed-circle files retain the shared code-46 layout; five CenterArc and one Sector corpus roots evaluate live. |
| Ratio | Partial: signed three-point projection plus first-segment/second-segment length ratio, live dependency updates, SVG, and single- or multi-page authoring. Generated positive and negative point-ratio files opened in Sketchpad 5; eight imported segment-ratio roots evaluate live and remain byte exact. |
| Value of Point | Partial: original and transformed segment/circle hosts, constructed straight paths, rays, lines, three-point arcs, circle-arcs, polygons, live Locus/ParameterLocus/FunctionPlot/sampled-mapping paths, and matching sector-boundary points. Constrained values, arbitrary-point path projection, precision 0–5, calculations, iteration depth, and save/reopen are supported. Six transformed/constructed path originals/edits display the expected native values and iteration counts and save byte-identically. Native ray/line and both arc measurement groups match generated groups byte for byte before label placement. Direct constrained points on point-target loci retain their normalized geometric value or canonical parameter value (radians for degree drivers); direct FunctionPlot constrained points report canonical function x, including asymmetric ranges and undefined-function recovery. See `dev/transformed_point_values_analysis.md`, [projection evidence](dev/point_value_projection_analysis.md), [function value evidence](dev/function_points60_native_analysis.md), and [Native curve parameter evidence](dev/path_curve_points58_native_analysis.md). |
| Coordinates | Partial: Cartesian point pair updates after drag, saves/reopens, generated file opened and saved in Sketchpad 5; legacy two-byte rich-text cache tails and supported constructed/colorized/iterated point sources import byte-exactly; complex names/formats pending |
| Abscissa (x) | Partial: Cartesian x value updates after drag, saves/reopens, generated file opened and saved in Sketchpad 5; optional native captions and observed display/cache formats, including nonzero presentation tails, import and feed calculations. Strict legacy code 125 imports as radius regardless of grid form, updates live, and feeds calculations; an authored code-125/code-124 pair opened and resaved byte-identically in Sketchpad 5. |
| Ordinate (y) | Partial: Cartesian y value updates after drag, saves/reopens, generated file opened and saved in Sketchpad 5; optional native captions and observed display/cache formats, including nonzero presentation tails, import and feed calculations. Strict legacy code 124 imports as polar angle regardless of grid form, displays in the document's degree/radian unit, updates live, and feeds calculations. |
| Coordinate Distance | Partial: Cartesian two-point value updates after drag, saves/reopens, generated file opened and saved in Sketchpad 5 |
| Slope | Partial: Cartesian slope of segments, rays, lines, constructed straight objects, transformed segments, and axes; precision 0–5, live calculations and iteration depth, import/save, and dependency remapping. Four generated originals/edits show the expected native slopes and iteration counts and save byte-identically. Native evidence is recorded in `dev/slope_hosts_analysis.md` |
| Equation | Partial: Cartesian equations of ordinary, parallel, and perpendicular infinite lines and coordinate axes, plus equations of original, segment-radius, and supported transformed circles; live geometry/frame edits, import/save, clipboard remapping, and native circle-label record preservation. Four circle originals/edits and six constructed-line/axis probes display the expected native equations and save byte-identically. See `dev/circle_equation_hosts_analysis.md` and `dev/constructed_line_equations_analysis.md`; non-square frames and other equation forms pending |

## Number

| Command | Status |
| --- | --- |
| Number Menu &gt; New Parameter | Partial: default t₁/t₂ parameters and multiple named ASCII parameters in a supported single-page sketch, including default parameters after a free point; finite values with up to 32 native numeric tokens, 0–5 display decimals, and scientific captions at magnitudes >10⁶; larger and finer-decimal parameter files and edits cross-open byte-identically; Unicode names and additional numeric layouts pending |
| Number Menu &gt; Calculate | Partial: source-free expressions using integers, decimals, unary signs, +, −, ×, ÷, powers, parentheses, functions, constants, and degree literals (up to 256 native tokens). Results retain scalar or degree units and can supply iteration depth or marked point rotation. Original small integer calculation records retain their exact format. A selected segment length, point distance, polygon perimeter, circle circumference, circle radius, or arc length can be multiplied by an integer from 0 to 99 or divided by a nonzero integer from 1 to 99; the result follows source edits and can supply a marked distance. Native factor-two files for all six sources, native length × 12, and native length ÷ 2/12 import without partial view and preserve their bytes. Generated factor-three, length × 23, and length ÷ 3/12 files opened, calculated correctly, and saved byte-identically in Sketchpad 5. Two selected length measurements or length calculations support +, −, ×, and ÷ in selection order, with cm for sums/differences, cm² for products, and unitless ratios; generated sums and negative differences update after moving either source and cross-opened byte-identically in Sketchpad 5. Length calculations can feed further ×/÷ calculations; generated chains recalculated after moving the source point, displayed 39.28 cm and 13.09 cm in native Sketchpad, and saved byte-identically. Selected parameters resolve page-generated names and literal-definition captions, including their live calculation bindings; native edited/copied files save byte-identically. A selected numeric parameter supports live expressions using its name, integer or decimal literals, +, −, ×, ÷, ^, and parentheses (up to 256 native tokens). A selected parameter-dependent calculation can also supply the next calculation's source value as `v`; native Sketchpad displayed 30.00 and then 42.00 after a parameter edit, saving both generated files byte-identically. The sibling native t₁ calculation chain imports without partial view and updates when t₁ changes. Point/parameter numeric iterations also accept scalar formulas with multiple inputs and a directly referenced mapped parameter at any source index; external values stay fixed within the sequence, all mapped references advance, and source edits refresh the sequence. Five generated/edited compound-formula iteration files displayed the native sequences and saved byte-identically. Parameter-calculation captions and attached/iterated numeric labels respect saved decimal precision; a zero-decimal a+0.5 iteration displayed 1, 1, 2, 2, 3 and saved byte-identically in native Sketchpad, confirming display rounding does not alter the iteration state. Degree literals such as `360°/n` preserve native angle units and display the degree suffix; chained expressions track scalar/degree units and reject incompatible arithmetic. The generated native regular polygon displayed 72.00° at n=5 and 60.00° at n=6. Selected three-point angle and arc-angle measurements now support expressions using `v`, including `v/2`, `v+10°`, and the scalar ratio `v/90°`; degree results can supply Mark Angle and rotation of supported points, segments, circles, and polygon interiors and follow geometry edits. Generated and moved angle-calculation files opened, displayed the expected values, and saved byte-identically in Sketchpad 5. New calculations are placed below numeric captions with room for fractions. Point ratios, slopes, Value of Point, scalar coordinate measurements (x, y, coordinate distance), literal integer calculations, and length-ratio calculations can supply the expression source `v`. Generated and edited scalar/coordinate examples opened and saved byte-identically in Sketchpad 5. A point-ratio calculation also supplies live carried-segment iteration depth; positive fractional depths truncate to whole steps within 1–128. Length and area measurements/calculations can now supply compound expressions using `v`, including decimal scaling, sums of compatible values, squared lengths, and unit-preserving quotients. Outputs track cm, cm², degrees, or scalar units; incompatible arithmetic is rejected. Length results supply marked distances and further length calculations. Generated and moved compound-length and circle/polygon-area files opened and saved byte-identically in Sketchpad 5. Built-in sin, cos, tan, asin, acos, atan, abs, sqrt, ln, log, sgn, round, and trunc support nested calls. Default trigonometric calculations use degrees for scalar and angle arguments; abs preserves units. Square roots support dimensional arguments: even unit exponents halve; odd exponents yield scalar values. ln, log and sgn return scalar magnitudes; round and trunc retain input units. Generated and edited scalar calculations, measured-angle functions, and length abs displayed correctly and saved byte-identically in native Sketchpad 5. Functions can supply live iteration depth; invalid domains suppress dependent geometry. Inverse trigonometric functions accept scalar arguments and return degree values that can supply marked point rotation and iteration. Constants pi/π and e are available in source-dependent expressions. Generated positive/negative inverse-function and constant files, a five-step inverse-angle segment iteration, and its out-of-domain source edit displayed correctly and saved byte-identically in native Sketchpad 5. Source-free constants/functions, signed powers, undefined expressions, and a constant-angle/five-step iteration also displayed correctly and saved byte-identically in native Sketchpad 5. Two to 256 selected numeric sources now support compound expressions using v1…v256 in selection order, with a/b aliases for the first two. Mixed compatible units, all source dependencies, live edits, copy/paste, deletion, and native save/reopen are supported. Native Sketchpad displays and saves three-scalar formulas, combined-length mean/product/rounded ratio, and mixed angle/scalar rotation and iteration before and after source edits byte-identically. Dimensional square roots, fractional powers, and integer unit powers now preserve native semantics, including cm⁴/°² captions and generic native quantity tags. Native creation fixtures import and save unchanged. Original and edited area-root chains, marked-distance/live-depth iteration, and length/angle roots and powers cross-opened and saved byte-identically. Higher units survive precision changes, copying, and page copying. Source-dependent dimensional exponents now produce scalar results, as in native Sketchpad. Computed/grouped/negative exponents also discard units; integer literal powers retain them. Native creation fixtures cover area, length, angle, and syntax distinctions. Scalar live depth and numeric iteration follow exponent edits and undefined-domain recovery. Unlike quantities add/subtract by magnitude and return Scalar, including literals and compound quantities. Native creation controls and original/edited 18-object API outputs prove values, units and exact saves; see [mixed arithmetic evidence](dev/mixed_quantity_arithmetic_analysis.md). Length, Area and Degree exponent sources and typed literal exponents now return Scalar magnitudes; pixel calculation literals convert into centimeters. Native-created controls and original/edited actual API writer outputs prove values, undefined domains and exact saves; see [quantity exponent evidence](dev/quantity_exponents_analysis.md). Referenced supported function definitions and derivatives can now be called using f1(…), f2(…), … at their selection indexes, including constant/nested calls, live function edits and numeric iteration labels. Typed Degree arguments follow the document angle mode; incompatible reference roles and invalid domains are rejected. Native-created controls and actual original/edited 28-object API outputs prove calls and iteration results; see [function call evidence](dev/function_applications_analysis.md). Referenced definitions now supply these calls and refresh their numeric coefficients during mapped iteration; function-call result-unit propagation remains partial. Other measurement sources and expressions beyond the supported bounds remain pending |
| Number Menu &gt; Tabulate | Partial: one-column default t₁ table with cached values from −99.99 to 99.99 and a later source ID after a free point, named ASCII parameter tables (including a later source ID), and two-column t₂/t₁ table with up to 256 rows and values from −99.99 to 99.99 in both columns, including parameter IDs after a free point; original and generated mixed-width tables cross-opened. Native numbered scalar tables with separate subscript text runs now import, render live values, move, change snapshot rows, and retain source names and table fonts. Two actual edited writers opened and saved byte-identically in Native with visible t₄/t₆ captions and values; see [indexed table evidence](dev/indexed_parameter_tables_analysis.md). Creating numbered scalar tables now atomically materializes unnamed page parameters, restores verified Tahoma 18 caption bounds, advances the page counter, and inserts the Tahoma 14 table font. Five actual creation writers, including t[1], a multi-digit index and a negative row, saved byte-identically in Native. Default t₁ and named ASCII creation also materialize the other unnamed page parameters atomically. New single-column tables resolve their Tahoma 14 font through the drawing namespace, including pending font additions; named snapshot edits preserve the actual font reference. Named tables now accept zero-precision integer rows and UTF-8 negative rows, and creating from a clean imported source marks the document dirty before save verification; see [named row evidence](dev/named_parameter_table_rows_analysis.md). Other naming fonts and scientific captions remain guarded. Literal-definition tables with typed unit captions remain unsupported. See [single-column transaction evidence](dev/default_parameter_table_transactions_analysis.md). Other sources and layouts pending |
| Number Menu &gt; Add Table Data | Partial: add default t₁ or named-parameter snapshot rows, or t₂/t₁ rows up to 256; fractional t₁ snapshots and named tables preserve earlier rows while appending the live value; generated t₁ and pair tables opened and saved in Sketchpad 5; other table kinds pending |
| Number Menu &gt; Remove Table Data | Partial: remove last default t₁, named-parameter, or t₂/t₁ row; earlier pair-table caches and plotted points stay fixed, and a generated four-row result opened and saved byte-identically in Sketchpad 5; other table kinds pending |
| Number Menu &gt; New Function | Partial: arithmetic expressions in `x` with integer or decimal constants, parentheses, unary signs, `+ − × ÷ ^`, and `sin`/`cos`/`tan`/`asin`/`acos`/`atan`/`abs`/`sqrt`/`ln`/`log`/`sgn`/`round`/`trunc`, plus π/e constants, decode, evaluate, render, and save. Nested built-in calls are supported; a generated square-root, absolute-value, natural-log, and common-log compound plot displayed correctly and saved byte-identically in Sketchpad 5. Double-clicking a displayed formula edits its expression and updates dependent plots; edits that change token length preserve the saved function wrapper. New and edited trig expressions offer the radians/degrees choice; the document angle unit supports undo/redo and save/reopen. Native-created six-function controls and actual original/edited 38-object API outputs verify primitive tokens, constants, rounding ties and inverse trig in both angle modes; see [primitive function evidence](dev/function_primitives_analysis.md). Signed caption coordinates preserve negative and large positions. Selected numeric sources and other supported function definitions now form live referenced definitions, with ordered v1/f1 aliases, mixed and nested calls, editing, plotting, undo/redo and exact saves. Actual original/edited 38-object API outputs verify live dependencies, radian conversion and mapped iteration coefficients; see [referenced function evidence](dev/function_references_analysis.md). Function-call result-unit propagation, further wrappers and longer expressions remain pending |
| Number Menu &gt; Define Derivative Function | Partial: a selected function definition or derivative produces a code-78 derivative with a symbolic expression. Native identity, square, cubic, sine, and degree-sine derivatives import and round-trip; generated square and sine derivatives in radians and degrees opened, displayed, and saved byte-identically in Sketchpad 5. Editing a source formula refreshes its derivative cache and dependent plots; an edited cubic source cross-opened. A degree-based trigonometric source offers the radians/degrees choice. Constant and variable exponents, the supported arithmetic/trigonometric grammar, and abs/sqrt/ln/log chain rules work. Undefined derivative domains suppress dependent plot segments. Generated x^x and edited ln(sqrt(x)) derivative formulas and curves displayed correctly and saved byte-identically in Sketchpad 5. Referenced-definition, inverse trig, sgn/round/trunc symbolic derivatives and longer expressions remain pending |
| Number Menu &gt; Define Function from Drawing or Picture | Partial: one selected independent, axis-aligned PNG picture can define a native code-122 function from the top edge of its alpha mask. The command creates a default coordinate system when needed, evaluates the edge in graph coordinates with the native default smoothing value 3, and writes the native record family. An actual eight-object writer opened with its picture-function caption and saved byte-identically in Sketchpad 5. Drawing sources, rotated/sheared pictures, non-alpha edge extraction, smoothing properties, approximate derivatives, and direct graphing remain pending. See [data-function evidence](dev/data_function_analysis.md) |

## Graph

| Command | Status |
| --- | --- |
| Graph Menu &gt; Define Coordinate System | Partial: default frame; selected origin, circle, one defining distance, or two defining distances with optional origin; finite segments and supported Length quantities, including typed parameters and calculations, work as selected defining distances. Stored numeric axes also accept Scalar, Degree, Area and Compound magnitudes using Native's centimeter scale. Zero, negative and undefined live unit values hide dependent geometry and remain saveable; restoring positive values restores the grid. Actual API outputs verify these states and unit families; see [coordinate unit evidence](dev/coordinate_units_analysis.md). Circle and distance axis ticks and labels render in native/SVG. Native examples decode; generated square, rectangular, and one-distance Polar frames opened with their grids and saved byte-identically in Sketchpad 5. Broader axis and grid-form editing pending |
| Graph Menu &gt; Mark Coordinate System | Partial: selecting an axis, origin, unit point, or square/Polar grid intersection marks its frame for later plots and measurements; the page link survives undo, save, and reopen; an older-frame Cartesian plot opened and saved byte-identically in Sketchpad 5; native grid-click marking changed the same link |
| Graph Menu &gt; Grid Form | Partial: Square, Polar, Rectangular, and trigonometric X-axis formats decode and save; polar rings/rays, independent vertical spacing, and π labels render in native/SVG. One-distance frames convert among Square, Rectangular, and Polar while preserving whether the Y unit follows the defining distance (code 56) or has its own spacing (code 57); two-distance frames convert between Rectangular and elliptical Polar. Generated conversions cross-open byte-identically. Two-distance Square behavior and broader axis formatting pending |
| Graph Menu &gt; Show Grid | Partial: coordinate-system visibility decoded and saved; hidden frame components omitted from native/SVG rendering; original and generated hidden frames reopened in Sketchpad 5 |
| Graph Menu &gt; Dotted Grid | Partial: original grid flag decoded, native and SVG dots rendered, undo/redo and save/reopen tested; generated Cartesian and Polar dotted frames opened and saved byte-identically in Sketchpad 5 |
| Graph Menu &gt; Snap Points | Partial: page flag decoded and saved; free-point creation and drag snap to the nearest Cartesian or Polar grid intersection; visible grid remains drawn while snapping, matching the original; original drag results matched numerically, and port-generated file opened, snapped again, and saved in Sketchpad 5; other point kinds pending |
| Graph Menu &gt; Plot Value on Axis/Path | Partial: selected default or named parameter plots as a live code 95 point, while the value dialog plots a fixed numeric code 123 point; X/Y axes, segments, rays, lines, circles, and polygon boundaries decode, render, and save. Scalar calculations can drive code 95 polygon points; independently verified code 123 literals use normalized perimeter fractions with wrapping. Fixed rotated/scaled ray and line hosts also support both record families: negative line positions extrapolate, while negative ray positions stay undefined with their marker segments. Authored, edited, and copied graphs save byte-identically; see [transformed path evidence](dev/transformed_straight_plot_analysis.md). Literal points retain labels and fonts through edits, copying, and new label creation; see [label evidence](dev/labeled_literal_path_analysis.md). Original and generated cases opened and saved in Sketchpad 5; further paths pending. Shared constructed and transformed straight hosts now use their evaluated affine basis and extent: segment outside values and negative ray values remain undefined with dependent measurements; full lines extrapolate. Native38 controls are byte-identical; see [straight path evidence](dev/value_straight_paths59_analysis.md). Scalar code95 values now locate points on point-target geometric and parameter loci using source-specific parameters and bounds; undefined targets preserve their dependent segments. Native flag4 hides the plotted point while retaining its geometry and observers. Code123 now also follows intrinsic sampled curves and ordinary point-output Locus paths using their verified normalized fractions; ParameterLocus and FunctionPlot literal forms remain guarded. See [literal locus evidence](dev/literal_locus_points_analysis.md). |
| Graph Menu &gt; Plot Points/Plot as (x, y)/Plot as (r, θ) | Partial: supported one-page sketches with an active coordinate frame can plot entered Cartesian or polar coordinates, including multiple points. Original files with a parameter, free point, or text as the first object decode and round-trip; their generated point groups match. Generated mixed-layout and multi-point sketches opened and saved byte-identically in Sketchpad 5. Literal plotted points depend only on their frame, retain supported labels/fonts/empty attributes, and follow frame drags and live unit edits. Their counted reference packet no longer creates a false dependency on the first page object. Unrelated first-point deletion and label editing survive strict save/reopen. Multi-page plotting and broader source layouts pending |
| Graph Menu &gt; Plot Table Data | Partial: one- to 256-row Cartesian or polar plot from the supported t₂/t₁ table, including values from −99.99 to 99.99 in both columns and a table and frame after a free point; duplicate rows combine, distinct rows produce separate points, and parameter edits update the live table row while plotted points stay fixed. A supported table can be plotted again after an edit. Original fractional and distinct-row polar files decode and round-trip; generated five-distinct-row, repeated-plot, shifted-ID polar, and mixed-width Cartesian files cross-opened byte-identically. Other column sources pending |
| Graph Menu &gt; Plot Function/Plot New Function | Partial: clicking a function formula selects its definition; Plot Function reuses that definition without adding a second one. Cartesian arithmetic expressions and the shared 13 built-in functions, plus π/e constants, in `x` plot and update after origin drag. Square-root and logarithm plots omit undefined-domain segments. A derivative can also be plotted; a generated square derivative plot opened and saved byte-identically in Sketchpad 5. Double-clicking a plot edits its numeric domain and sample count; original and newly created range edits cross-opened byte-identically. Tangent and reciprocal curves leave asymptote gaps in native/SVG rendering and hit testing. Original trig, compound, decimal, and reciprocal formulas decode; generated existing-definition, tangent, reciprocal, sine, combined trig, compound, and decimal plots opened and saved byte-identically in Sketchpad 5. Supported referenced definitions also plot through live numeric and nested function sources, including inherited reciprocal pole gaps. Other sources, function types, arrowheads, and advanced discontinuities pending |
| Graph Menu &gt; Plot Parametric Curve | Partial: two selected or chosen functions, including supported referenced definitions and derivatives, plot a rectangular or polar curve over an editable numeric domain. Native code-126/127 files import as editable documents; formula edits update their curves. Double-clicking a plot edits its numeric domain and sample count; original and newly created range edits cross-opened byte-identically. Generated rectangular, polar, custom-domain, and derivative-source files opened, rendered, and saved byte-identically in Sketchpad 5. The port renders curves in native UI and SVG. Function types beyond the current expression parser, arrowheads, and advanced discontinuities pending |

## Action Buttons

| Command | Status |
| --- | --- |
| Hide/Show | Partial: native buttons for one, two, and three targets import as editable. Native three-point and mixed point/segment/polygon buttons retain selection order. Rust-generated versions opened and saved byte-identically in Sketchpad 5; native clicks hid all targets and changed the action to Show while leaving untargeted construction points visible. The port now adds buttons to supported existing multi-page documents using page-local target IDs; page-2 and both-page generated files cross-opened byte-identically, and a native click on the page-2 button hid only its targets. A native page-2 button confirmed the same reference rule. Copied native two-target buttons also cross-open. Further target combinations and native copy placement pending. Options 0–3 execute fixed visibility; 4–7 alternate the next action. Caption metadata is preserved, variable-length updates save safely, and hidden Hide/Show children can run through Presentation. See [visibility evidence](dev/visibility_options48_analysis.md). |
| Animation | Partial: buttons for one parameter or one to seven source points import, create, save, and start or stop motion without the floating controller. Direct point targets include free, transformed, straight, circle, arc, polygon-boundary, supported point-curve, and sector-boundary points. The 32 corpus buttons targeting transformed points import and run with temporary, nonserialized motion state. Corpus-observed opaque mode tags are accepted only for their matching point/parameter layouts and survive settings edits. Selecting a segment, line, ray, or circle creates a two-source button; triangles through heptagons create three- to seven-source buttons. Targets use native reverse source order. A segment with hidden source points also works. Native files for all these shapes import; all ten port variants opened and saved byte-identically in Sketchpad 5. Native clicks on generated segment, quadrilateral, and hexagon buttons moved every source point; the same playback was checked in the Rust editor. The port now adds Animation buttons to supported existing multi-page pages with page-local targets and settings; page-2 point and segment buttons plus a both-page file cross-opened byte-identically, and a native click on the generated page-2 segment button moved both endpoints without changing page 1. Native mixed free/segment/circle buttons also import and cross-open. Parameter buttons support continuous or discrete changes, rate, step, and numeric domain; native custom files import, and a port-generated discrete button cross-opened with the expected properties. Other indirect constructions, exact timing, and larger target sets pending |
| Movement | Partial: ordered multi-pair buttons import and save with optional absent labels and empty metadata. Creation uses selected source/destination pairs, and shared movement values support free/constrained points, supported graph controls, and editable numeric Parameters. Source construction and unit are retained; point targets project onto their host and numeric targets copy magnitude across units. Native controls establish reverse pair order for Instant Follow point targets, frozen initial targets, and sampled numeric targets. Activation has one undo checkpoint. Existing multi-page page-local references remain supported. Parameter editing retains the document safety guards. Exact motion rates, broader graph-unit policies, and Native polygon point trailing metadata remain pending. Evidence: [movement controls](dev/movement_controls46_analysis.md), [action records](dev/action_records46_analysis.md). |
| Presentation | Partial: all corpus-observed native Presentation settings import as editable and round-trip byte-identically. The port creates simultaneous or sequential buttons for selected actions and runs Hide/Show, Animation, Movement, Scroll, Link, and nested Presentation actions. The dialog and runtime support Deselect Objects, Stop Animations, and Erase Traces before starting; simultaneous stopping after the last action, first action, or elapsed time; and a timed pause between sequential actions. Native property probes establish the flag bits and tenths-of-a-second encoding. The independent native progress/state word is preserved but not assigned unproved resume semantics. Clicking again advances sequential execution or stops simultaneous children. Both basic port-generated modes and supported multi-page page-local references cross-opened and ran in Sketchpad 5. Additional child combinations and native progress-state behavior remain pending; see [Presentation evidence](dev/presentation_button_analysis.md). |
| Link | Partial: native default, custom, and even-length URL buttons import as editable and round-trip byte-identically. The port creates URL Link buttons, renders them, and opens HTTP(S), file, and sibling `sketchdoc://` links; a generated URL file cross-opened byte-identically, and a Rust click followed a sibling document link. Native Page Links to pages 1 and 2 import losslessly, and editor clicks switch to their saved destinations. The port creates Page and URL Links on any page of supported multi-page documents. Generated files from both the first and last pages opened and saved byte-identically in Sketchpad 5; native clicks navigated to their target pages. Native Page Links that activate a destination Hide/Show button import, create, and run in the Rust editor; a port-generated file opened and saved byte-identically in native Sketchpad, and clicking it there hid the destination point. The Rust editor can save the resulting Hide/Show state in a multi-page file; that file opened and saved byte-identically in native Sketchpad. Native URL buttons with four-byte text padding and opaque action fields retain their layout. Dragging URL and Page Link captions updates both their action anchor and saved bounding box; edited files opened and saved byte-identically in Sketchpad 5. See [link layout evidence](dev/link_caption_layout_analysis.md). General multi-page editing, other destination action types, and other special URL schemes pending |
| Scroll | Partial: native Upper Left and Center buttons import as editable and round-trip byte-identically. The port creates either mode, draws and activates the button, and pans action buttons with the viewport and SVG/PDF export. Generated one- and two-button files opened and saved byte-identically in Sketchpad 5. Clicking the port-generated Upper Left button there placed its point at the canvas corner; a desktop click did the same and moved the button out of view. New buttons can be added to supported existing multi-page pages with page-local point references; generated page-2 and both-page files cross-opened byte-identically. A native click on the generated page-2 button moved its point to the canvas corner without changing saved bytes. Broader point/document layouts pending |

## Custom Tools

| Command | Status |
| --- | --- |
| Create New Tool | Partial: selected supported single-page constructions become registered embedded tools with independent point givens or a consumed path/finite-segment boundary, local reference remapping, preserved drawing state, and structural undo/redo. The actual API writer opened and saved byte-identically in Native; the created tool also executed there. Selection must include the actual givens and construction; native-created and API-authored path/segment tools have been invoked in Native. A selected scalar or degree parameter, or segment-length measurement, with its direct supported calculation can also become a generic numeric tool. The numeric writer preserves separate native script fonts and drawing state; broader numeric graphs, other given categories, and definition editing remain pending. See [creation evidence](dev/tool_creation_analysis.md), [boundary evidence](dev/tool_authoring_boundaries_analysis.md), and [numeric authoring evidence](dev/tool_numeric_authoring_analysis.md) |
| Use a Custom Tool | Partial: point, polymorphic path, finite segment, generic numeric, distance-valued, and Function givens execute with ordered bindings, live dependencies, hidden helpers, and undo/redo. Path recipes materialize points on segments, rays, lines, circles, arcs, and polygon boundaries; independent Native controls verify the shared given category. Square and segment/midpoint API outputs opened and saved byte-identically in Native. Segment givens accept ordinary and transformed finite segments. Numeric inputs specialize calculations from actual source units; scalar, length, angle, and measured-length API outputs cross-open byte-identically. Native gives legacy subtype51 the same generic Measurement Value prompt and subtype48 a distinct Distance Measurement Value prompt. Abstract calculations also preserve expression-declared Function versus Numeric source roles, including the corpus 35-source function-call family. Native explicitly prompts to match Function `h` for subtype 57; a newly complete function-and-plot tool executes with a live function, four Parameters, and a coordinate system and reparses exactly. Numeric output captions use the current verified calculation default and follow the last caption at its current position. Fresh measured-area and length-product writers cross-open byte-identically with separated live captions. Single and nested formula fractions use native expression-tree heights, including 24Bold source captions, with three exact writer proofs. Native also establishes subtype-79 and subtype-81 as scalar- and angle-Parameter-specific givens, and subtype-15 as an arbitrary-polygon given. Native subtype identity plus exhaustive consumer roles establish subtype-39 as a coordinate-system given; the port imports it without inventing axes and matches only an actual active-page frame. Polygon/scalar/angle-Parameter known prototypes remain partial, while the newly complete coordinate prototypes still require broader numeric-caption placement to materialize. Generated/named fraction source expansion, broader measurement captions and font preferences, native reuse of existing constructions, and other given categories remain pending. See [runtime evidence](dev/tool_runtime_analysis.md), [legacy numeric-given evidence](dev/tool_legacy_numeric_givens_analysis.md), [angle-Parameter evidence](dev/tool_angle_parameter_given_analysis.md), [function-given evidence](dev/tool_function_given_analysis.md), [function-calculation evidence](dev/tool_function_calculation_analysis.md), [scalar-Parameter evidence](dev/tool_scalar_parameter_given_analysis.md), [polygon evidence](dev/tool_polygon_given_analysis.md), and [coordinate-system evidence](dev/tool_coordinate_system_given_analysis.md) |
| Document tools | Partial: embedded prototypes have a separate object/font namespace and remain byte-preserved through supported drawing edits; picture-bearing prototypes resolve their document-global PNG resources in preserved source order. Registered complete definitions permit structural drawing edits and supported point/path/segment/numeric/distance/Function-input execution, plus bounded creation. Scalar-Parameter, angle-Parameter, Polygon, CoordinateSystem, Distance, and Function givens import with strict matching categories; incomplete prototypes and separately guarded output layouts still block materialization where applicable. Definition editing and other input categories remain pending. See [namespace analysis](dev/embedded_tool_scope_analysis.md), [picture-resource analysis](dev/tool_picture_resources_analysis.md), [legacy numeric-given evidence](dev/tool_legacy_numeric_givens_analysis.md), [angle-Parameter evidence](dev/tool_angle_parameter_given_analysis.md), [function-given evidence](dev/tool_function_given_analysis.md), and [coordinate-system evidence](dev/tool_coordinate_system_given_analysis.md) |
| Tool Folder tools | Partial: tools from top-level `.gsp` files in the selected folder appear in a separate menu section, disambiguated by source file. They support ordered selected inputs and click-driven Point, Path, Segment, Polygon, Numeric, Distance, Function, scalar-Parameter, angle-Parameter, and CoordinateSystem inputs through immutable source snapshots and the existing transactional external-prototype runtime. Numeric categories and Functions use caption hit testing; coordinate systems use grid hit testing and the same active-page type gates as document tools. Recursive discovery, native ordering/grouping, automatic refresh, broader input families, and proof of the destination namespace against Native remain pending. See [tool-folder evidence](dev/tool_folder_analysis.md) |
| Choose Tool Folder | Partial: opens the platform folder picker, atomically installs a path-sorted catalog of readable top-level `.gsp` documents, reports skipped unreadable files, and preserves the catalog across New/Open without creating drawing history. Cancel and load errors preserve the previous catalog. The choice is session-only; Native restart persistence and example-document generation remain pending |
| Forget Tool Folder | Partial: clears the session catalog and cancels an active folder-tool binding without altering already materialized geometry or drawing history. Native preference cleanup across restarts remains pending |

## Tool and document workflows

| Workflow | Status |
| --- | --- |
| Select and drag points | Partial: existing free points and points constrained to segments, rays, lines, circles, arcs, or polygon boundaries can be moved and saved in supported multi-page files. A segment-point move matched native Sketchpad byte for byte; circle and arc moves cross-opened and saved byte-identically in native Sketchpad. Moving a first-page free point together with adding a Page Link also cross-opened byte-identically |
| Point tool | Partial: free points and points on existing segments, rays, lines, circles, arcs, and polygon boundaries can be added on supported multi-page pages; generated files opened and saved byte-identically in Sketchpad 5 |
| Segment tool | Partial: segments built from existing or newly created points on supported multi-page pages; generated object group matches native Sketchpad byte for byte and the file cross-opened |
| Circle tool | Partial: circles built from existing or newly created points on supported multi-page pages; generated object group matches native Sketchpad byte for byte and the file cross-opened |
| Text, image, table, iteration, plotting | Partial: literal text, point labels, a single PNG picture, Cartesian `x`/`x²` plots, and a bounded point translation iteration cross-open. Literal captions with saved bold, italic, underline, and size records import as editable text; edits in mixed-style two-caption sketches cross-opened byte-identically. Bold, Italic, and Underline formatting of free text also cross-opened. A numeric parameter, calculation, coordinate pair, or supported nested numeric label can be attached to a point with the native code-90 label record; it follows point movement and source edits. Generated moved labels cross-opened byte-identically, and imported labels retain observed calculation-cache `?3`/`?4` presentation containers. Other text formatting and iteration forms pending |
| Import, export, print | Partial: single PNG import, SVG/PDF export, and Linux printing for supported visible geometry; other image formats and physical print verification pending |
| Multi-page documents | Partial: pages can be added, copied, renamed, removed, and reordered in supported sketches; generated copied, named, removed, and reordered files opened and saved byte-identically in native Sketchpad. Page navigation, supported constructions and measurements, Copy/Paste/Cut/Clear, existing-point movement, visual styles and tracing, Hide/Show state, and Page/URL Link insertion work on supported pages. Existing scalar parameter values and their supported live table rows can also be edited on known-object pages, with strict source-record checks and atomic failures. Parameter/table pages can be copied and reordered while retaining their local source IDs. Native edited, copied, and reordered files saved byte-identically; see [multi-page parameter evidence](dev/multi_page_parameter_values_analysis.md). Broader object families and document options remain pending |
| Cross-open generated files in Windows Sketchpad 5 | Partial: nine-object file edited and saved in Sketchpad 5, then edited in Rust and reopened under Wine |

Live scalar iteration depths truncate positive fractions before applying the supported 1–128 bound: 5.5 produces five steps. Undefined, non-scalar, or out-of-range depths suppress repeated geometry. Fixed dialog depths remain whole numbers.

Source-free calculations now also support typed centimeter lengths, area products,
scalar functions of radian literals and inverse-angle functions. Expression edits
retain their quantity unit and update dependent marked-distance transforms with
undo/redo and save/reopen. Original corpus controls and an authored file saved
byte-identically by native Sketchpad are documented in
[the calculation evidence](dev/source_free_quantity_calculations_analysis.md).

Many-input calculations preserve input order and now accept up to 256 inputs and
256 expression tokens, with exact native index validation. Original 20-input
formulas import and evaluate; 51-input authoring, live parameter edits, dependent
length transforms, undo/redo and save/reopen have a native-saved writer control.
See [many-input evidence](dev/many_input_calculations_analysis.md) for scope.

Marked distances also accept unitless Scalar Parameters and calculations,
interpreted as signed centimeters. Point/segment/circle/polygon polar images,
Cartesian and mixed images share this conversion. Typed-Length arithmetic keeps
its unit checks. Native measured controls, actual writer proof, editor marking,
live edits and iteration regressions are recorded in
[scalar marked-distance evidence](dev/scalar_marked_distances_analysis.md).

Recursive polygon reflection sources now use the shared polygon predicate and
vertex count instead of requiring an original literal polygon. An original
compound-reflection drawing and three actual API files prove source and mirror
edits, independent vertex coordinates, exact Native saves and strict reopen.
See [polygon source evidence](dev/transformed_curves45_analysis.md).

Stored Point construction application now evaluates input substitution through
one shared DAG definition. Native proof distinguishes reevaluated carried nodes
from frozen omitted dependencies, and actual public API authoring passes Native
open/display/save controls. Nested iteration contexts, reference remapping,
cascade deletion and undo/redo are covered; numeric seeds and non-Point outputs
remain unsupported. This does not add custom-tool invocation or authoring.

Point on Object supports default, circle-derived, distance-derived and
trigonometric coordinate axes. Native subtype38 uses a fixed span centered at
the origin; creation, dragging, save/reopen and Value of Point use the same
parameter conversion. General point/polygon Value of Point is available through
existing measurement commands in either selection order; constrained polygon
points and literal/value path points also work. Source edits remain live.
See [axis evidence](dev/graph_axis_point49_native_analysis.md) and
[polygon value evidence](dev/numeric_roots49_analysis.md).

Live Color from Values commands support Spectrum, Grayscale, RGB and HSV for
Point, Segment, Ray, Line, Circle, Polygon and Arc, including a whole selected
batch in one undo step. Numeric channels retain selection order and stay live
after edits and reopening. The shared source geometry supports constraints and
other dependents; native ranges preserve Clamp/Cycle/Reflect semantics. Current
menu creation uses range0–1 with reflection; the API also exposes custom ranges.
Spectrum uses interpolated Native palette samples, without pixel parity claims.
See [color model evidence](dev/colorized_elements50_analysis.md) and
[Native writer evidence](dev/color_wrappers50_native_analysis.md).

Attached numeric labels now display live typed measurements with the source's
precision and units. Bare and literal-cached native numeric references preserve
cached records and independent font indexes. Attach Selected Value to Point
accepts measured quantities as well as parameters/calculations. Unsupported
math macros and nonnumeric anchored text remain opaque. Scalar iteration depth
accepts all supported Scalar values through the same unit classifier, including
ratios and path values. See [label evidence](dev/point_numeric_labels50_analysis.md)
and [depth evidence](dev/scalar_iteration_depth50_analysis.md).

Referenced captions now preserve a strict rich-text tree and resolve ordinary
slots through live object names, numeric values and action captions. Live Caption
from Selection creates a bound caption; move, select, font edits, SVG/canvas and
save/reopen use the shared text paths. Single-click referenced action text runs
the existing handler even when its source button is hidden. Cached spelling and
opaque allocation bytes survive saves. Flagged name/expression forms and math
subtrees retain unsupported status; encoded text colors are preserved without
pixel parity claims. See [Native caption evidence](dev/referenced_rich51_native_analysis.md).

Generic carried iteration supports PointNumericLabel through existing
IterationBinding outputs, with live scoped values/anchors, frozen omitted
sources, undefined numeric text and final-only output. Fonts inherit from the
source label without creating an editable font record on the binding. Exact
legacy numeric iteration packets retain their specialized model and value-series
semantics. Rendering, selection, editing/undo and dependency remapping have
focused regression and actual API Native writer controls. See
[Native carried-label evidence](dev/mapped_numeric_labels51_native_analysis.md).

Point on Object also reads host subtype46 for Parallel and47 for Perpendicular,
using the existing centered line parameterization. Drag/save keeps the original
subtype on single and multiple pages. Polygon boundary packets contain a u16 edge
index followed by six opaque bytes; edits preserve those bytes. Actual public
movement API files reopen and save exactly in Native. Bisector subtype44 now shares
the proved parameter frame with canonical13: vertex plus15sqrt10 times the sum of
unit arms, scaled by the stored parameter. Zero or opposite arms are undefined;
valid dependent measurements retain their definitions. Actual API single/multiple-page
dragging and undefined-source saves cross-open byte-identically in Sketchpad5. See
[Bisector parameter evidence](dev/bisector53_native_analysis.md).
See [host evidence](dev/path_host_subtypes52_native_analysis.md) and
[polygon API evidence](dev/polygon_constraint52_api_native_analysis.md).

Native109/110 one-literal, one-live-coordinate points share a CoordinatePoint
definition containing NumericExpr coordinates and a frame reference. Scalar
or Degrees values, negative coordinates, independent rectangular frame units, live source
edits, fixed-coordinate API edits, undo and linked measurements use the common
dependency graph. The public authoring API and preserving single/multiple-page
writer are covered; there is no new desktop creation command. Mapped Scalar
iterations reuse the same point evaluation and freeze omitted source boundaries.
Actual API Native controls prove both live and frozen iteration results. Native
degree-backed code-110 points use the displayed degree magnitude, consistent
with ordinary numeric XY points. Length and area sources remain guarded. See
[coordinate API evidence](dev/coordinate_points52_api_native_analysis.md) and
[degree-coordinate corpus evidence](dev/corpus_coverage.md#degree-valued-fixed-coordinate-points).


Ordinary code35 loci also traverse supported numeric intermediates. Native
PointValue→Calculation→numeric-dilation point curves and primitive-segment sweeps
retain exact full ancestor refs and reevaluate each driver sample. Source edits,
creation, undo and deletion have focused tests; three actual public construct_locus
files cross-open and save byte-identically in Sketchpad5. Other ranges/targets remain guarded.
See [numeric locus evidence](dev/locus_numeric53_native_analysis.md).

Inherited class1/subtype89 vector-transformed points reuse the generic element and
transform model when the saved source header proves that identity. Mapped-source
and chained point packets retain own styles, live edits and exact saved records.
See [inherited point evidence](dev/inherited_point_transform53_native_analysis.md).


Point-locus imports retain both proved 0902 word32 values0/1 and the captured
intermediate ancestry order. Decoding and edited-save validation check the exact
role prefix, terminal target and ancestor multiset, preserving opaque range bytes.
Five Native field-only controls and three public move/edit/restoration files save
byte-identically with positively checked live curves. New authoring keeps its
canonical order/flag. See [retained locus metadata](dev/locus_metadata54_native_analysis.md).

Arc and ray/line outputs share a native transform operation parser. Vector,
reflection and fixed-polar straight chains support live dependencies, parameter
constraints, desktop commands, picking/rendering and source degeneration/restoration.
Native independent midpoint and support observers verify the parameter frame and
direction. GraphAxis is admitted only for the separately proved fixed/numeric
rotation forms; its other straight operation families remain guarded. See
[Native straight chains](dev/straight_transform54_native_analysis.md).


The eight generic straight public authoring/edit controls, including a source
moved into an undefined zero ray, reopen and save byte-identically in Sketchpad5.
Independent point and support observations confirm both ray/line parameter frames
and directions. The port preserves valid definitions through degeneration and undo;
a malformed canonical straight output cannot be reinterpreted through a legacy
point/circle decoder fallback. See [public API evidence](dev/straight_transform54_native_analysis.md).


Checkpoint55 adds intrinsic point access to single-branch mapped iteration
results, using the shared mapped evaluator. Imported aliases and public API
authoring expose the final carried output, including original source position
at numeric depth0 and undefined terminal geometry without prefix fallback.
Final-only rendering does not change this point. Typed undefined endpoint
segments remain savable. Caller substitutions and frozen external inputs are
preserved; ordinary binding image behavior is unchanged. The original-layout
Native controls retain an out-of-range borrowed font selector as read-only
metadata; direct API controls use valid formatting and provide separate
creation/edit/save verification. See [endpoint evidence](dev/iteration_endpoint55_native_analysis.md)
and [shared prerequisite audit](dev/numeric_shared_roots55_analysis.md).

The final full corpus scan recovers2,560 drawing slots, including458 calculations,
while all1,599 originals remain byte-identical. The scoped core/editor/numeric/
coverage gates pass994 tests. These are measured implementation checks; the
release command gate remains unmet. Copying and custom-tool work remain deferred.


Checkpoint56 adds directed polygon boundary construction and command support
for ordered Segment/Ray/Line parents in either selection order. One intrinsic
point Kind searches positive carrier events independently of the source extent,
retains undefined definitions, and supports live edits, undo, deletion and
page-local save. Repeated/retraced polygon runs remain ordered vertex sequences.
Twenty-four Native controls and24 direct public API files match byte-for-byte.
See [boundary command evidence](dev/polygon_boundary56_analysis.md).

Inherited point89 Scalar dilation33 imports now normalize to the existing
generic transform definition while preserving source records and styles.
Actual source identity, operands and payload guards remain strict. Seven
Native controls prove canonical4 equivalence, frozen inputs, edits and defined
zero/negative ratios. Quantity and inherited rotation cases remain guarded.
See [Scalar evidence](dev/inherited_numeric_transform56_native_analysis.md).

The final full scan restores1,779 drawing objects, including194 calculations,
plus seven incidental prototype objects, with all1,599 originals byte-exact.
Scoped checks pass1,004 tests (829 core,137 editor,34 numeric,4 coverage).
These measured checks do not satisfy the remaining release command gate.
Copying and custom-tool implementation remain deferred.

Checkpoint57 extends the intersection command to FunctionPlot and point-target
Locus/ParameterLocus curves, normalizing selections to the proved carrier/curve
roles and creating every ranked crossing. Curve tangencies, gaps and undefined
sources retain their evidenced behavior. Arc/Circle and Arc/Arc selections also
use the existing13/14 constructions, with native circular and bounded linear
arc rules. Direct API61 curve files and18 round-pair files match Native saves
byte-for-byte. The tests cover live edits, dependent observers, undo/deletion,
scoped evaluation and page-local curve references. Calculation readers/writers
accept up to4,096 tokens while retaining the existing256 source limit and strict
units/schema guards; six Native controls include two unmodified originals.

The full scan restores1,344 drawing slots, including304 calculations, and nine
additional documents become fully decoded. All1,599 originals remain
byte-identical. These checks do not establish the remaining release command
requirements. Copying and custom-tool work remain deferred.

Final checkpoint57 scoped gates pass1,019 tests (841 core,140 editor,34 numeric,
four coverage). New page-local intersections retain imported curve sources;
new plot authoring keeps its existing single-page restriction.


Checkpoint61 broad prerequisites recover1,386 drawing objects and retain all1,599 original files byte-for-byte. Shared sampled DAG102 evaluation, extended35 constrained drivers and the proved89 prefix profile are independently checked in Native. Scoped gates pass1,077 tests (891 core,148 editor,34 numeric,four coverage). Imported sampled paths support PointOnObject selection, projection/drag, undo/delete and strict save/reopen; this does not add a sampled-DAG authoring dialog. Copying and custom-tool authoring remain deferred; release command requirements remain unmet. See [current corpus evidence](dev/corpus_coverage.md).

Checkpoint62 implements the authorized format/IR/engine/editor workspace split. Object identity and semantic transform models live in IR; the engine derives dependency indexes from construction relationships and retains scoped evaluation and lossless source saving. The root package preserves public module paths and supports a headless probe. Workspace gates pass1,136 test executions, including duplicated renderer assertions; the full1,599-file scan retains checkpoint61 coverage and exact original bytes. This migration adds no command parity claim. See [crate boundaries and verification](dev/crate_layers62_analysis.md).


Checkpoint62 subsequently implements shared FunctionPlot/SampledMapped PointOnCurve drivers for geometric loci, reusing the existing construction, evaluation, rendering, dragging, undo/deletion and save/reopen paths. Independently proved outer numeric domains differ from direct source queries; sampled-Circle open/closed metadata is retained. Ten Native controls and nine actual API constructions are whole-file exact;120 observer roles pass. Full scan restores99 drawing slots, including57 loci, and five additional documents decode completely. All1,599 originals remain byte exact; workspace gates pass1,143 executions. This adds bounded locus-driver support, not arbitrary capture/range/target admission or deferred copying/custom-tool authoring. See [implementation and gates](dev/locus_point_sources62_analysis.md).


Checkpoint63 extends geometric-driver Locus through the shared affine/Colorized straight model: Segment/Ray/Line targets and generic finite Segment hosts retain live dependencies, per-sample color, independent carrier extent, pen editing, undo, and strict save/reopen. SVG/canvas/picking consume the engine's sampled carriers. Deleted unsaved additions are compacted without evaluating removed sources. Surviving plot/function embedded-reference remapping remains an explicit unsupported deletion case.

The actual scan recovers118 drawing slots, including94 loci; all1,599 originals retain entire bytes. Workspace gates pass1,155 executions;30 independent Native exact files include12 entirely API-authored writer controls. Native stored sample hints3..5 have four display intervals; exact raster budgeting is not a requirement and a general rounding rule is not inferred. Copying and custom tools remain deferred. These checks do not satisfy the full release command gate. See [implementation and evidence](dev/locus_generic_straights63_implementation.md).


## Checkpoint64 shared rich reference roles and automatic names

The shared engine now evaluates supported73 Value/A/B/E references and90 point-anchored text/function/numeric contents with live/frozen source semantics, strict dependency guards, font editing, deletion/undo and save/reopen. Anonymous Calculation/Ratio/Length Name requests use verified active-closure/object/operand order and0967; assigned names survive hiding and dirty saves persist07d5 rather than recomputing names from remaining visibility.

Actual scan recovers1,378 drawing slots (379 code73,999 code90); unsupported16,683→15,305, drawing-complete542→573, all1,599 originals whole-byte exact. Workspace1,172 test executions and headless build pass; three initial entirely API-authored Native controls are final-library exact, and eight Name writer controls pass strict physical-label checks plus independent Native exact saves and painted witnesses. Rich math/font layout, additional source roles and nested annotation iterations remain bounded gaps. Copy/custom tools stay deferred; this does not satisfy the full release command gate. See [implementation](dev/referenced_shared64_implementation.md).

## Checkpoint65 shared filled-disk iteration

Mapped CircleInterior bindings share scoped evaluation, analytic geometry, fill presentation, picking and undo/save services. Native seven-state controls verify live/frozen Circle hosts, final-only output, undefined final members and recovery. Actual corpus recovery is192 drawing objects; all1,599 originals remain byte-identical and1,176 workspace test executions pass. All nine fresh public writer controls save whole-file exact in Native; independent opacity witnesses confirm three coincident frozen samples composite separately. Sector transformations, general nested capture and new multi-page iteration authoring remain unfinished. See [implementation](dev/circle_interior_iteration65_implementation.md).

## Checkpoint66 shared anchored text and nested terminal scopes

Mapped anchored Value references reuse live/frozen text resolution, inherited source fonts and shared picking/rendering. Nested point terminals distinguish carried descriptors from structural bindings and frozen entry environments, with local seed shadowing and no fallback from an undefined terminal. Complete active capture is validated by decode, authoring and strict save; partial factor/image capture remains a distinct unfinished evaluation-phase contract.

Workspace1,190 test executions and headless build pass. Actual recovery is338 drawing objects, including196 code77 and75 code89; unsupported15,113→14,775 and drawing-complete584→604. All1,599 originals remain whole-byte exact. Six entirely public rich writer controls pass Native exact saves; eight public nested controls pass Native exact saves and independent geometry checks. Rich Name/Definition/Expression capture, general nested collections and broader page authoring remain unfinished. See [implementation](dev/shared_scopes66_implementation.md).

## Checkpoint67 scoped Locus collections

Straight-output Locus bindings share the mapped evaluator's outer scope and retain a local sweep domain. SVG/canvas/picking and selected Scalar iteration authoring use the returned independent strokes. Eight original-role Native controls verify live/minimal capture, edited parameters/depth, undefined gaps, direct sampled colors and transformed binding color. Eight entirely public writer inputs reproduce those Native inputs byte-for-byte and also save whole-file exact in Native; two more public final-only controls show only the last generation's five members. The workspace passes1,199 executions and the headless build succeeds. The corpus recovers38 drawing objects while all1,599 originals remain whole-byte exact; unsupported slots14,775→14,737. Direct binding of an inner mapping root remains unsupported: the attempted control crashed and actual source-role scanning found no matching original binding. General collection and procedural inner-mapping phases remain pending. See [implementation](dev/scoped_collection67_implementation.md).

## Checkpoint68 sampled point-curve collections

Code102 sampled-curve sources can now be carried into scoped code77 bindings over outer point iterations. A parent-scope regression proves that the local sweep shadows only its own seed and preserves unrelated outer substitutions. Public authoring, styled SVG in a complete document, imported-document picking/visibility/undo, and strict save/reopen pass. Original and publicly edited circular-chain controls save whole-file exact in Native; visibility-only controls isolate binding23's right-green geometry, whose edited 1,311 pixels are all within 1.5px of the model strokes (maximum 0.705px). The original file remains a partial view and its SVG export is still blocked by the existing safety guard.

Workspace gates pass1,206 test executions and the headless build succeeds. The actual corpus recovers352 object slots; unsupported14,737→14,385, all-kind complete593→598, and drawing-complete604→609. All1,599 originals remain byte exact. Other code102 source profiles, structural partial views, copying and custom tools remain open; this does not establish complete command parity. See [implementation and Native evidence](dev/sampled_collection68_implementation.md).

## Checkpoint69 FunctionPlot in mapped constructions

A FunctionPlot used upstream of a mapped point or shape now retains its function/frame recipe and resolves each dependency in the current scope. When the plot wrapper is omitted from carried objects, a carried frame still moves per generation; an omitted frame remains at its entry value. This was established by four entirely public Native controls and an actual triangle-iteration original plus public edit, all six saved whole-file exact. Native point and segment witnesses corrected an initial 36–37px scope error; the final original three-point errors are below0.62px. A direct FunctionPlot binding remains outside the proved source roles.

Workspace gates pass1,211 executions and the headless build succeeds. All1,599 originals remain whole-file byte exact. Unsupported slots fall14,385→14,226, including27 code89 mapping roots and95 code77 binding descendants; all-kind complete598→601 and drawing-complete609→612. The broader command gate remains unmet, with copying/custom tools still deferred. See [implementation and original Native evidence](dev/mapped_function_plot69_implementation.md).

## Checkpoint70 point-output Locus collections

Point-output Locus sources now replay their local driver sweep within each outer mapped scope. The existing code89 root carries the Locus recipe, while its code77 binding presents a separate styled trace for each generation through SVG, canvas and picking. The directly visible source Locus remains independent. A Native original, two visibility controls and parameter edit all save whole-file exact; hiding the binding isolates three generated triangles from a fourth direct-source triangle exposed by the edit. Two entirely public writer states also save exact and show the expected live 40px extension. The isolated Native trace pixels are within 1.32px of the modeled geometry.

The workspace passes1,216 test executions and the headless build succeeds. All1,599 originals remain byte exact. Unsupported slots fall14,226→13,965, including50 code89 roots and98 code77 bindings; all-kind complete601→625 and drawing-complete612→636. Nested Locus paths and circle/arc output collections remain separate; copying and custom tools remain deferred. See [implementation and Native evidence](dev/scoped_point_locus70_implementation.md).

## Checkpoint71 referenced π suffix and numeric subscript

Two bounded Native rich-text operators now share the existing referenced-text AST and live numeric resolver. `<!70>` prints a π suffix after a cached value reference; active `<-...>` presents a one-character base with a live numeric subscript inside a vertical label. The strict parser retains unknown-token rejection outside these proved structures. Native original/show/hide controls isolate `−1π` versus `−1` and `A₃`; separately labeled one-byte parent-reference controls preserve stale caches but display `1π` and `A₅.₀₀`. Seven valid Native states save entire files exactly. Clean editor controls cover SVG, picking, live value/precision edits, undo and save/reopen; original partial-view SVG guards remain.

Workspace gates pass1,224 test executions and the headless build succeeds. All1,599 originals remain byte exact; unsupported slots fall13,965→13,794, including35 code73 captions,46 code77 bindings,16 code48 calculations and nine code89 mappings. All-kind complete625→626 and drawing-complete636→637. Other rich operators and broader source roles remain separate; copying and custom tools remain deferred. See [implementation and Native evidence](dev/referenced_token71_implementation.md).

## Checkpoint72 grouped unary referenced functions

The shared numeric/symbolic expression parser now accepts a unary call whose grouped operand continues with an infix operator, as in the Native calendar's `trunc((Y−1)/x)`. Its code71 definition and downstream code48 calculation decode and evaluate live; strict closing-token and source-role guards remain. The original remains partial because of unrelated records. Native whole-file exact saves and clearly labeled field-level synthetic controls verify the calendar calculation at 2024 and 2025; Native's infinity display at `x=0` is still outside the engine's finite-value presentation API.

The final scan preserves all1,599 originals byte-for-byte and restores623 object slots, reducing unsupported13,794→13,171. All-kind complete rises626→633 and drawing-complete637→646. The headless build and1,373 scope-aware workspace test executions pass; the unrestricted gate has seven failures in deferred copying/custom-tool tests. This is decoding and live evaluation progress, not full command parity. See [implementation and evidence](dev/referenced_function72_implementation.md).

## Checkpoint73 standalone code26 points

Code26 `CustomTransformPoint` now decodes only the Native-proved standalone Free-point/marked-angle/marked-length profile, using the shared polar transform expression and dependency evaluator. Direct code102 trace endpoints, Segment outputs, Scalar operands and groups with extra label records remain opaque. Two independent originals plus positive, negative and zero-distance controls demonstrate live positions and whole-file exact saves. Native has not established fresh code26 authoring, so the writer refuses new instances of this flavor.

The final scan preserves all1,599 originals byte-for-byte and restores129 slots, including50 code26 roots and downstream transforms/bindings; unsupported falls13,171→13,042. All-kind complete rises633→636 and drawing-complete646→649. The headless build and1,376 scope-aware tests pass; seven copying/custom-tool tests remain excluded by the user's scope. See [implementation and Native evidence](dev/code26_standalone73_implementation.md).

## Checkpoint74 Vector-transformed code26 sources

Standalone code26 Points now also accept a Vector-transformed Point source through the same live polar expression. An independent Native original and four whole-file exact controls isolate the marker and confirm it follows a `(40,20)` source move. The saved move is field-level synthetic because an unrelated partial-view interior guard rejects the public edit. The final scan restores96 further slots while preserving all1,599 originals byte-for-byte; unsupported falls13,042→12,946. Complete-document counts remain unchanged. The headless build and1,377 scope-aware tests pass. Other transformed source flavors and code102-linked programs await their own proof. See [implementation and evidence](dev/code26_transformed74_implementation.md).

## Checkpoint75 high-word referenced-function wrappers

Code71 expression wrappers now accept upper count words 1 or 2 only when the low half exactly matches the token count and every existing wrapper, syntax and source-role check passes. Editing preserves the high half. Native high1 controls save whole-file exact and show a plotted function moving with its numeric source; high2 controls retain the function packet and live parameter behavior, although Native normalizes unrelated metadata on save. The two imported files remain partial, so their source edits are explicitly synthetic controls. The corpus remains byte-exact for all1,599 originals and restores28 slots, lowering unsupported12,946→12,918. The headless build and1,379 scope-aware tests pass; seven copying/custom-tool tests remain excluded by the user's scope. See [implementation and Native evidence](dev/referenced_function75_implementation.md).

## Checkpoint76 FixedPolar source for standalone code26 points

The shared code26 polar operation now accepts a FixedPolar-transformed Point as its source. Native original/show/upstream-move controls isolate the output marker and prove its `(40,20)` movement; all four saves are whole-file exact. The source move is field-level synthetic because unrelated partial-view validation blocks the public edit's save. The final scan restores52 slots, including51 code26 roots, while preserving all1,599 originals byte-for-byte; unsupported falls12,918→12,866. The headless build and1,380 scope-aware tests pass. MarkedPolar and other point-source flavors, code102-linked programs and Segment outputs await separate proof. See [implementation and Native evidence](dev/code26_fixedpolar76_implementation.md).

## Checkpoint77 MarkedPolar source and segment PointValue boundary

Standalone code26 Points now also accept a MarkedPolar-transformed Point source. Fifteen Native-saved states are whole-file exact; isolated marker controls prove source movement and live angle changes. The angle controls exposed a separate PointValue rule: a PointOnSegment measured against another Segment clamps to `[0,1]`, while independent Free-point projection remains unbounded. The imported original is partial and its saved upstream edits are field-level synthetic. The final scan preserves all1,599 originals byte-for-byte and restores68 slots, lowering unsupported12,866→12,798; one more document becomes complete. The headless build and1,382 scope-aware tests pass with seven copying/custom-tool tests excluded. See [implementation and Native evidence](dev/code26_markedpolar77_implementation.md).

## Checkpoint78 scalar coordinate marker

Code65/66 scalar coordinate measurements now accept one empty `232e` after the normal label/style pair, under strict class, reference, format and record-order checks. The marker remains in the saved bytes. A visible code65 original and same-length point edit save whole-file exact and show live x values `−1.48→−1.00`. A paired code65/66 original retains both markers and references through Native save, though Native normalizes unrelated bytes and its x/y captions cannot be read independently. The final corpus scan preserves all1,599 originals byte-for-byte and restores1,017 slots, lowering unsupported12,798→11,781. The headless build and1,385 scope-aware tests pass, excluding seven copying/custom-tool tests under the user's scope. See [implementation and Native evidence](dev/coordinate_measurements78_implementation.md).

## Checkpoint79 analytic sector rotation

Class-13/subtype-68 code27 fixed rotations now decode when an intrinsic SectorBoundary and Free center have the proved two-reference layout. The object retains its live source, matrix expression and analytic sector geometry; fill and picking use evaluated geometry. Six Native-saved controls are whole-file byte exact and isolate both a 180°→90° angle change and upstream motion. All1,599 corpus originals still round-trip byte-exact; unsupported slots fall11,781→11,764 (17 code27 objects), and1,386 scope-aware test executions pass. Reflected/chained sectors and transformed centers remain unproved, so the auditor's 194-node reach is only a frontier. See [implementation](dev/sector_transform79_implementation.md).

## Checkpoint80 reflected sector chain

An intrinsic SectorBoundary can now reflect about a Segment, and that reflected Sector can undergo a fixed rotation about a Free point. The parser requires the two exact Native record layouts and rejects unproved marker-bearing and alternate-mirror groups. Seven Native-saved controls are complete-byte exact and isolate both operations through visibility differences; edited controls are field-only synthetic because the original document is partial. The strict final corpus scan keeps all1,599 originals byte exact and lowers unsupported11,764→11,745 (18 code34 reflections and one chained code27 rotation). The Sector-focused suite and1,387 scope-aware test executions pass. Further mapping and command behavior remain open. See [implementation](dev/sector_transform80_implementation.md).

## Checkpoint81 repeated sector rotation

A fixed-rotated Sector can now be rotated again about a Free point using the same exact code27 record layout and ordered semantic matrix sequence. Native controls isolate the second 30° object and show that changing either its own angle or the upstream 30° angle yields the same 90° sector geometry; every tested full-file save remains exact. The compiled corpus checks all1,599 originals byte-exact and restores160 code27 objects, lowering unsupported11,745→11,585; 1,388 scope-aware test executions pass. Sector mapping and remaining geometry families are still outside this proof. See [implementation](dev/sector_transform81_implementation.md).

## Checkpoint82 imported polygon-walk points

Imported subtype-15 Polygon boundaries now keep their wire identity for ordered walks beyond nine stored vertices. Under exact point group layouts, a subtype-15 PointOnPolygon uses its stored edge slot and live fraction, including repeated source references. Eight-slot, repeated twelve-slot, and high-word-1 266-slot Native controls isolate the visible point through fraction and source changes; all 26 tested whole-file saves are byte exact. The corpus restores 290 slots behind 14 points, lowering unsupported11,585→11,295 across all1,599 byte-exact originals; 1,392 scope-aware test executions pass. Extra-marker point groups remain unsupported. See [implementation](dev/polygon_walk82_implementation.md).

## Checkpoint83 intrinsic circular-segment boundary

Exact no-marker code83 groups now import as live CircularSegmentBoundary objects: the source Arc sweep closes with its endpoint chord to form the filled region. Fourteen Native saves, including visibility pairs and an upstream endpoint edit, are whole-file byte exact; isolated gray fill follows the chord and arc. The compiled corpus keeps all1,599 originals exact and restores102 code83 objects, lowering unsupported11,295→11,193; 1,395 scope-aware test executions pass. A point constrained to this boundary and six alternate code83 record layouts remain unsupported pending their own contracts. See [implementation](dev/circular_segment83_implementation.md).

## Checkpoint84 directed angle values

Standard three-point `090e` groups now import as two distinct live Degrees values. Native edits show code115 uses the positive signed turn modulo 360°, while code116 uses its reverse; the unchanged downstream labels read `146.34→207.27°` and `34.02→320.84°` respectively. Eight Native full-file saves are exact. Across all1,599 byte-exact originals, 36 drawing slots and five isolated prototype slots decode, lowering total unsupported11,193→11,152; 1,397 scope-aware test executions pass. Alternate marker packets, downstream label objects and authoring remain separate. See [implementation](dev/angle_values84_implementation.md).

## Checkpoint85 named angle aliases

Strict code120 aliases of verified angle-value parents now form a generic numeric alias node with a live Degrees source. Native hiding controls attribute the changing `∠ABC` and `D` captions to the aliases, independently of parent angle markers; two further precision controls show the saved `0903` decimal field controls displayed digits, and all twelve whole-file saves are exact. The compiled 1,599-file scan restores 302 drawing and 11 prototype slots through 31 direct aliases and their existing downstream graph, lowering unsupported11,152→10,839. All originals remain byte-exact, and 1,400 scope-aware workspace test executions pass. Other alias source categories and new-object authoring remain open. See [implementation](dev/angle_alias85_implementation.md).

## Checkpoint86 colorized filled disks

Strict code75 wrappers over `CircleInterior` now share the existing numeric color evaluator and filled-region presentation. Native source motion, visibility isolation, Spectrum and Grayscale scalar substitutions establish a live disk carrier; all eleven complete saves are byte exact. Across 1,599 byte-exact originals, 25 drawing disks and two prototype slots decode, lowering unsupported10,839→10,812; 1,404 scope-aware workspace test executions pass. Disk transforms and fresh disk-color authoring remain outside the verified contract. See [implementation](dev/colorized_disk86_implementation.md).

## Checkpoint87 three-point angle markers

Strict code113 groups now derive one or two live arc strokes from three point references. Their code120 aliases reuse the numeric-alias object, calculate a signed angle, and display degrees or radians according to the page setting. All 23 Native controls save byte exact; the compiled scan restores 160 drawing slots, lowering unsupported10,812→10,652 across 1,599 exact originals, and 1,411 scope-aware workspace test executions pass. Alternate marker flags and unproved alias parents remain unsupported. See [implementation](dev/angle_marker87_implementation.md).

## Checkpoint88 mapped colorized disks

The shared code89 mapped DAG now carries a bounded `Colorized(Disk)` and code77 binds its per-generation filled outputs separately from circle outlines. Native six-state visibility controls save whole-file exact; earlier scalar and movement controls prove changing colors and positions. The corpus restores 64 slots, lowering unsupported10,652→10,588 across 1,599 exact originals. Imported mapped value depth 359 is evaluated with generation caps; fresh authoring retains its existing limit. Source-interior hiding has no valid Native control. See [implementation](dev/colorized_iteration88_implementation.md).

## Checkpoint89 colorized polygon regions

Strict code75 class4/subtype15 polygon wrappers retain live vertices, numeric color bindings and independent visibility. Generic code89/code77 mapped bindings now evaluate and draw each polygon generation with its own color. Native direct/mapped visibility, range, point-motion and depth controls save exactly in 11 states. The corpus restores 66 drawing slots, lowering unsupported10,588→10,522 across 1,599 byte-exact originals; seven more documents decode fully. Polygon source hiding and other packet variants remain unverified. See [implementation](dev/colorized_polygon89_implementation.md).

## Checkpoint90 imported URL buttons with opaque suffixes

Native URL Link actions with observed 44- and 56-byte payloads can carry nonzero bytes after the terminated URL. The importer keeps those bytes opaque while preserving the existing strict action and label checks. Eight passive Native controls save whole-file exact; a safe local-link click gives no reliable suffix activation rule. Changing an imported nonzero-suffix URL is refused, while ordinary label and document edits preserve it. The compiled corpus restores 408 buttons and 241 dependent texts, lowering unsupported10,522→9,873 across 1,599 byte-exact originals. See [implementation](dev/url_suffix90_implementation.md).

## Checkpoint91 imported multi-resource pictures

Independent code0 picture objects can now select one of several validated embedded PNG resources by native index and retain their own affine transform and visibility. Five-record and empty-marker six-record groups use the existing image renderer/SVG/picking path. Six Native visibility/translation controls save whole-file exact. Multi-picture files remain structurally read-only; two labeled image groups are still opaque. The corpus restores 146 drawing slots, lowering unsupported9,873→9,727 across 1,599 byte-exact originals. See [implementation](dev/image_objects91_implementation.md).

## Checkpoint92 colored free-text runs

Ordered native `R/G/L` channel triples with values1..256 now color individual text runs in canvas and SVG output. Seven Native controls save whole-file exact and separate the date-run palette from unchanged preceding text; zero-channel inheritance remains unverified for import. The compiled corpus restores 83 text slots, lowering unsupported9,727→9,644 across 1,599 byte-exact originals. Mixed-format content replacement retains its existing guard. See [implementation](dev/rich_style92_implementation.md).

## Checkpoint93 referenced-function π suffix

The shared expression parser now accepts the exact native `π,0x0100` pair, restoring 216 code71 function objects across the corpus without a new object variant. Six Native controls preserve whole files byte-for-byte and show that the ordinary upstream numeric chain changes while the function caption remains symbolically displayed. The engine predicts a changing `f97(1)` value; no direct Native function-value readout was available. Unsupported falls9,644→9,428 across 1,599 exact originals. This example remains a partial document with expression edits guarded. See [implementation](dev/referenced_function93_implementation.md).

## Checkpoint94 point-anchored pictures

The initial strict code99 admission covered zero-rotation images using their live point parent and stored positive scales around the PNG center. Free and constructed parent changes update canvas, SVG, selection geometry, and picking through the existing image object. Sixteen Native controls save whole-file exact; an isolated coin PNG moves (+40,+20) exactly with its point. The compiled scan restored 97 image objects across 44 files, lowering unsupported9,428→9,331 while all1,599 originals stayed byte exact. The later rotated-attachment extension is recorded below. Resource-table structural editing remains guarded. See [implementation](dev/anchored_image94_implementation.md).

## Checkpoint95 nested polar point transforms

Strict code26 point transforms now accept an already decoded `DualMarkedPolar` point as their source. Their own angle and length operands remain live, and both transforms compose in the shared DAG evaluator. Ten Native movement, length and angle controls save whole-file exact. The compiled scan restores 105 drawing slots including 38 direct roots across six files, lowering unsupported9,331→9,226 while all1,599 originals stay byte exact. Other source point flavors remain outside the verified admission. See [implementation](dev/point26_dual95_implementation.md).


## Imported two- and three-point pictures

Strict code85 rectangle pictures and code100 parallelogram pictures now use
their live point parents through the shared image transform, canvas, SVG,
picking, and iteration paths. The remaining 39 code108 crops resolve against
live picture and polygon parents. All 136 drawing attachments and all 39 crops
decode; 115 downstream objects follow, lowering unsupported slots 4,311→4,021. All 1,599
corpus files remain byte-exact. Imported resource tables remain read-only, and
fresh attachment authoring is not yet implemented. See [picture evidence](dev/image_analysis.md).

## Imported locus variants

Native code-35 loci now retain sparse and independent-target-branch captures,
either observed presentation word, and filled-region targets beyond polygons.
Point-output sampled objects can serve as bounded curve-collection targets.
Records whose
base driver remains `Free` apply their explicit stored path only in the sampled
locus scope; validation rejects a path that depends on that driver. Ordinary
public locus construction remains restricted to constrained points. The
compiled corpus restores 107 loci and 47 downstream objects, lowering
unsupported slots 4,021→3,867 while all 1,599 originals remain byte-exact.
This import expansion does not imply arbitrary locus authoring or full command
parity. See [locus evidence](dev/locus_midpoint_analysis.md).

## Native vector-translated generic polygons and pictures

Code-16 vector translation now accepts generic class-4/subtype-15 polygons
only when their source has the same native header. It also resolves the
observed class-16/subtype-78 picture profile from a code-99/code-100 source,
two live point endpoints, an exact four-record layout, and a PNG dimension
cache repeated twice. Picture rendering composes the source placement with the
live vector displacement. The compiled corpus restores 47 code-16 objects and
26 downstream objects, lowering unsupported slots 3,867→3,794 while all 1,599
originals remain byte-exact. This strict import support does not add fresh
translated-picture authoring or mutable resource tables. See
[polygon evidence](dev/vector_polygon_translation_analysis.md) and
[picture evidence](dev/image_analysis.md).

## Native affine picture transforms

Class-16/subtype-78 picture outputs now share the typed transform model and
compose their source placement with live vector/marked translations,
point-angle or numeric rotations, fixed/numeric dilations, and reflections.
Import requires a decoded picture source, validated operand roles, the bounded
record family, and a duplicated source-PNG dimension cache. The compiled scan
restores 37 transform roots and 22 downstream objects, lowering unsupported
slots 3,794→3,735 while all 1,599 originals remain byte-exact. Transformed crop
objects and fresh picture-transform authoring remain outside this admission.
See [picture evidence](dev/image_analysis.md).

## Rotated point-attached pictures

Code-99 point attachments now retain any finite native rotation together with
their positive horizontal and vertical scales. The pixel affine is centered on
the PNG before rotation and then follows the live point anchor. Import also
recognizes the exact hidden internal `[1]` metadata record found in two corpus
files without exposing it as an editable geometric label. This restores four
picture roots and one dependent object, lowering unsupported slots
3,735→3,730; all-kind-complete documents rise 1,167→1,170 and
drawing-complete documents rise 1,196→1,199. All 1,599 originals remain
byte-exact. Fresh attached-picture authoring and resource mutation remain
guarded. See [picture evidence](dev/image_analysis.md).

## Native angle marker presentation

Three-point angle objects now share live sector fills, fixed-radius arc marks, right-angle squares, optional arrows, stored bisector label placement, and picking across codes 113, 115, and 116. Arrow flags and opacity survive clipboard copies and serialization. Labeled and empty-232e profiles restore 159 additional corpus slots; all 1,599 original files remain byte-exact. Numeric angle conventions remain live through their existing DAG references. Exact font and arrowhead rasterization remains outside this verification. See [implementation and native controls](dev/angle_marker_presentation.md).

## Native rich-caption symbols and layout

The strict referenced-text AST now retains the observed `O1`–`O5` geometry
decorations, vertically centered groups, and five-field outer frame metrics.
Canvas and SVG math layout draw overbars, one- and two-ended arrows, and arc
marks instead of flattening them. The native integral and other observed
symbol-font codes decode to Unicode, including correction of code `103` to the
native minus sign. Reference role `D` is accepted only for numeric sources and
renders as a signed additive term from the live value.

Across the complete corpus this bounded work restores 259 slots, including 166
direct code-73 captions and their existing descendants. Unsupported slots fall
3,730→3,471; all-kind-complete documents rise 1,170→1,228 and drawing-complete
documents 1,199→1,258. All 1,599 originals remain byte-exact. Source-aware
resolution covers observed geometry values, names, definitions, and expressions.
Legacy Chinese markup uses a strict whole-markup GBK fallback. The 14 remaining
code-73 captions parse successfully but depend on unsupported parent objects.

## Parameter-driven geometric locus families

Code-97 parameter loci now share the geometric sampler used by ordinary loci
when their live target is a supported straight, filled region, or nested
point-curve collection. Admission requires an observed scalar/degree Parameter
or ParameterCalculation driver, the exact dependency capture, matching
unit/range metadata, and the target's native subtype. Unsupported-parent and
length-driver profiles remain guarded. The writer emits subtype 42 for point
traces and subtype 43 for geometric families; save/reopen preserves editable
ranges and stroke styles.

The complete corpus restores 89 direct code-97 objects and 19 existing
descendants. Unsupported slots fall 3,471→3,363; all-kind-complete documents
rise 1,228→1,243, drawing-complete documents rise 1,258→1,273, and partial views
fall 880→877. All 1,599 originals remain byte-exact.

## Arc and sector Radius measurements

Code-46 Radius now accepts the same supported circular geometry families as the
shared arc/sector evaluator instead of requiring a Circle object. Five native
CenterArc measurements and one SectorBoundary measurement have the established
code-46 numeric layout and evaluate to finite positive live radii. The editor
authors the same measurement for supported arcs and sectors, including on
multi-page documents; calculations and captions reuse the existing Length path.

The compiled corpus restores 97 drawing slots: six Radius roots and 91 existing
descendants. Unsupported slots fall 3,363→3,266, all-kind-complete documents
rise 1,243→1,248, drawing-complete documents rise 1,273→1,278, and partial views
fall 877→875. Embedded-tool counts are unchanged, and all 1,599 originals remain
byte-exact.

## Segment Ratio measurements

Code 47 now distinguishes both observed dependency forms. Three point
references retain the signed projection `(C−A)·(B−A)/|B−A|²`; two finite
Segment references evaluate as `length(first) / length(second)`. The second
segment must remain nondegenerate. The shared numeric path supplies scalar
formatting, captions, calculations, SVG, deletion propagation, and live edits.
The editor accepts two selected segments in numerator/denominator order, and
both single- and multi-page writers emit the native counted two-reference
layout without adding another binary template.

The corpus contains eight newly decoded segment-ratio roots. A runtime audit
evaluates all eight to finite positive values, including the π construction's
`3.1415926535898`. They unlock 68 existing descendants for 76 drawing slots
total. Unsupported slots fall 3,266→3,190, all-kind-complete documents rise
1,248→1,253, drawing-complete documents rise 1,278→1,283, and partial views
fall 875→871. Prototype coverage is unchanged at 610 unsupported slots; all
1,599 originals remain byte-exact.

## Sector interior name metadata

Native code-82 Sector groups may contain a bounded `07d5` name record and
indexed `07d8` text style even though the interior does not present an editable
geometric label. The decoder now validates and preserves those records using
the same inert-metadata policy as CircleInterior, while continuing to require an
earlier supported Arc, the exact class/subtype header, one counted reference,
and only the optional empty `232e` marker. A representative recovered sector
executes through the existing analytic boundary evaluator and round-trips
exactly.

The pass restores 16 drawing sectors and 51 drawing descendants, plus 12 sector
objects inside embedded-tool prototypes. Unsupported slots fall 3,190→3,111
(2,513 drawing and 598 prototype), all-kind-complete documents rise
1,253→1,256, and drawing-complete documents rise 1,283→1,286. Partial views
remain 871 because that count also includes structural editing restrictions.
All 1,599 originals remain byte-exact.

Native code-92 iteration value tables now support one or more ordered carried
numeric/function columns. Import and save preserve their anchor, visibility,
optional caption, font reference and observed empty marker; the editor and SVG
show the initial row followed by generation-numbered mapped values. A Native
`N!.gsp` run and the port both produce six rows ending with `7.00` and
`5040.00`, and an edit probe confirms anchor save/reopen. The compiled corpus
restores all 42 remaining code-92 drawing slots, lowering unsupported objects
2,223→2,181 and raising complete documents/drawings
1,326/1,358→1,346/1,381. All 1,599 originals remain byte-exact; no standalone
test case was added.

Native code-91 snapshot tables now support two to seven ordered numeric or
referenced-text columns. Import, copy/remap, movement, save, editor display, and
SVG preserve or present their rich headers, cached rows, units, formulas,
anchors, optional captions, font references, visibility, and alignment bytes.
Native checks cover the nine-row coordinate table and the zero-state
π-estimation table with its undefined cell. The compiled corpus restores all 40
remaining code-91 drawing slots, lowering unsupported objects 2,181→2,141 and
raising complete documents/drawings 1,346/1,381→1,358/1,393. All 1,599
originals remain byte-exact; no standalone test case was added.

## Circular-segment interiors and measurements

Circular-segment interiors now accept both observed native presentation profiles,
and their endpoint chord closes the source arc for analytic Area and Perimeter.
Six code-83 roots, two Area measurements, and three descendants restore 11
drawing slots. Unsupported falls 3,111→3,100, complete documents rise
1,256→1,259, and complete drawings rise 1,286→1,289. All 1,599 originals remain
byte-exact.

## Circle-and-endpoints Arc Length

Code-44 Arc Length now distinguishes the native three-reference circle/start/end
form from a one-reference arc host. Twelve roots and 61 descendants restore 73
drawing slots. Seven roots currently evaluate to finite positive values; five
preserve valid undefined geometry. Unsupported falls 3,100→3,027 (2,429 drawing
plus 598 prototype), complete documents rise 1,259→1,260, complete drawings rise
1,289→1,290, and partial views fall 871→870. All 1,599 originals remain
byte-exact.

## Colorized sector measurements and transformed-point animation

Area, Perimeter, and Arc Length accept every supported sector family, including
transformed and Colorized sectors. The compiled scan restores six drawing slots:
one direct code-42 Area and five already-supported descendants.

Code-62 Animation buttons may target a supported transformed point. Native GSP
was exercised under Xvfb/Wine: the target and its dependents moved, saving did
not alter its underlying free source, and Stop restored the construction-defined
position. The editor mirrors that with a temporary evaluator override excluded
from history, dirty state, and serialization. It restores 32 drawing buttons.
Together these passes lower unsupported slots 3,027→2,989 (2,391 drawing plus
598 prototype), raise complete documents 1,260→1,262 and complete drawings
1,290→1,292, and lower partial views 870→869. All 1,599 originals remain
byte-exact.

## Segment markers

Strict code-121 groups now draw one, two, or three fixed-size perpendicular ticks
on supported original and transformed Segment hosts. Flagged groups draw a
forward or reverse arrow parallel to the host. Native GSP under Xvfb/Wine
confirms the tick count, placement, length, spacing, arrow shape, and both arrow
directions. Imported styles, visibility, picking, dependency copies, and single-
or multi-page serialization use the ordinary stroke pipeline. The compiled scan
restores 27 drawing slots: unsupported falls 2,989→2,962 (2,364 drawing plus 598
prototype), complete documents rise 1,262→1,268, complete drawings rise
1,292→1,298, and partial views fall 869→867. One sampled-curve arrow remains
unsupported rather than being misclassified as segment geometry. All 1,599
originals remain byte-exact.

## General imported code-26 polar translations

The strict code-26 `[source, Degrees, Length]` operation now decodes native Point
and Segment outputs in both observed record layouts: unlabelled and labelled with
font metadata. Trace-linked points retain their own polar construction while the
later sampled mapping independently references them. Native Xvfb/Wine controls
cover labelled free and nested sources, a trace-linked source, and a Segment;
each Ctrl+S save is whole-file exact.

The compiled corpus restores all 20 typed drawing code-26 records and 75
dependents. Unsupported falls 2,141→2,046, complete documents/drawings rise
1,358/1,393→1,369/1,404, unsupported documents fall 241→230, and partial views
fall 840→839. All 1,599 originals remain byte-exact. Scalar-operand profiles and
embedded-tool code-26 records remain outside this admission; no standalone test
was added.

## General Plot Value sources and paths

Code-95 Plot Value accepts scalar, Length, Area, and Degrees sources on the
already-supported axis and geometric path families. Numeric points on a function
plot treat their source as the x-domain value. Sector boundaries use normalized
physical-perimeter parameters for both code-95 live values and code-123
literals, including structurally valid undefined states.

Native Xvfb/Wine controls establish quantity-axis placement, x=1.57 function
placement, and sector traversal. Freshly authored Length-on-axis and combined
live/literal sector files reopen and save whole-file exact in Sketchpad 5. The
corpus restores 23 direct roots and 67 dependents: unsupported falls
2,046→1,956, complete documents/drawings rise 1,369/1,404→1,380/1,415,
unsupported documents fall 230→219, and partial views fall 839→834. All 1,599
originals remain byte-exact; no standalone test was added.

## Duplicate carried references in mapped iterations

Native code-89 reference tails may repeat carried prerequisites or branch
images. Import canonicalizes that tail to the first occurrence of each object
before using the existing mapped-DAG validation and evaluation paths; source
records retain the exact native bytes, while fresh public authoring continues to
require unique carried IDs. The corpus restores 16 code-89 definitions and 59
code-77 bindings: unsupported falls 1,956→1,881, complete documents/drawings
rise 1,380/1,415→1,391/1,426, unsupported documents fall 219→208, and partial
views fall 834→831. All 1,599 originals remain byte-exact. Native Xvfb/Wine
opens, displays, and saves the 8,184-byte tiling control exactly; the port SVG
matches its repeated square-and-arc geometry. No standalone test was added. See
[analysis](dev/mapped_iteration_duplicate_carried_analysis.md).

## Nested mapped-iteration capture

An outer mapped iteration may carry an earlier complete mapped root and its
code-77 bindings as structural scope descriptors. Import requires ordered,
same-page objects, a valid bound source in the inner carried DAG, the explicit
inner root, and complete capture of every inner node depending on an outer seed.
The bindings do not become generic mapped values or outer binding kinds. The
corpus restores 11 code-89 definitions and 48 code-77 bindings: unsupported
falls 1,881→1,822, complete documents/drawings rise 1,391/1,426→1,401/1,436,
unsupported documents fall 208→198, and partial views fall 831→826. All 1,599
originals remain byte-exact. Native and port controls agree on the 3/9/18/30/45
pipe piles, and Native preserves the 5,682-byte source exactly. No standalone
test was added. See
[analysis](dev/mapped_iteration_nested_capture_analysis.md).
