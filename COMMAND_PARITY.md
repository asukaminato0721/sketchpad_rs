# Sketchpad 5 command parity

Source: the bundled Sketchpad 5 Reference Center menu pages. Status describes the current Rust port. **Partial** means a UI command exists but its full Sketchpad behavior and Windows cross-open have not yet been verified. **Pending** means absent.

The release gate is every sketch and document command verified after editing, saving, reopening, and cross-opening in Sketchpad 5. This gate is not met.

## File

| Command | Status |
| --- | --- |
| New Sketch | Partial |
| Open | Partial |
| Save | Partial |
| Save As | Partial |
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
| Copy | Partial: selected supported objects and required parents, including segment and midpoint cases observed in Sketchpad 5 |
| Paste | Partial: internal clipboard, 20-unit offset, and generated segment cross-opened in Sketchpad 5 |
| Clear | Partial: selected objects and dependent constructions in supported single-page files; generated result reopened in Sketchpad 5 |
| Action Buttons | Partial: Hide/Show button creation and interaction; other button kinds pending |
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
| Text | Pending |
| Hide Objects | Partial |
| Show All Hidden | Partial |
| Show Labels | Partial: point-label visibility through editor controls; other object labels pending |
| Label Objects | Partial: point labels can be named and saved; other object labels pending |
| Trace Objects | Partial: point and segment header flags, save/reopen, undo/redo, and bounded native drag trails; generated point and segment files opened and saved byte-identically in Sketchpad 5, with point trail visually confirmed |
| Erase Traces | Partial: clears native in-memory trails; original application's trail clearing verified for a port-generated traced point |
| Animate Objects | Pending |
| Increase Speed | Pending |
| Decrease Speed | Pending |
| Stop Animation | Pending |
| Hide/Show Motion Controller | Pending |
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
| Locus | Pending |

## Transform

| Command | Status |
| --- | --- |
| Mark Center | Partial: marked point can be used for later rotate and dilate commands; original marked-only file is byte-identical to its unmarked source |
| Mark Mirror | Partial: marked straight object can be used for later reflection; original marked-only file is byte-identical to its unmarked source |
| Mark Angle | Pending |
| Mark Ratio | Pending |
| Mark Vector | Pending |
| Mark Distance | Pending |
| Translate | Partial: point translation by fixed polar angle and distance cross-opened |
| Rotate | Partial: point around selected center by fixed angle; generated file opened, edited, and saved in Sketchpad 5 |
| Dilate | Partial: point and circle by fixed ratio around a center point; generated file opened, edited, and saved in Sketchpad 5 |
| Reflect | Partial: point and circle across selected segment or line; generated file opened, edited, and saved in Sketchpad 5 |
| Iterate | Pending |
| Define Custom Transform | Pending |
| Edit Custom Transforms | Pending |

## Measure

| Command | Status |
| --- | --- |
| Length | Partial: segment length, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Distance | Partial: two-point distance, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Perimeter | Partial: polygon interior perimeter, dynamic value, SVG, generated file edited in Sketchpad 5 |
| Circumference | Partial: circle circumference, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Angle | Partial: three-point angle, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
| Area | Partial: circle area, dynamic value, SVG, generated file opened and edited in Sketchpad 5 |
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
| Number Menu &gt; New Parameter | Partial: default t₁/t₂ parameters with editable −99.99 to 99.99 values at two-decimal precision; original fractional, negative, and two-digit files decode and generated files cross-open; arbitrary names and wider values pending |
| Number Menu &gt; Calculate | Partial: literal 2+3 expression, move and save/reopen; original file cross-opened |
| Number Menu &gt; Tabulate | Partial: one-column default t₁ table and two-column t₂/t₁ table with one or two rows, including fractional t₂ values; other sources and layouts pending |
| Number Menu &gt; Add Table Data | Partial: add default t₁ snapshot row or a second t₂/t₁ row, save/reopen; other table kinds pending |
| Number Menu &gt; Remove Table Data | Partial: remove last t₁ snapshot row, save/reopen; other table kinds pending |
| Number Menu &gt; New Function | Partial: Cartesian `x` and `x²` definitions save and cross-open; other expressions pending |
| Number Menu &gt; Define Derivative Function | Pending |
| Number Menu &gt; Define Function from Drawing or Picture | Pending |

## Graph

| Command | Status |
| --- | --- |
| Graph Menu &gt; Define Coordinate System | Partial: default Cartesian frame, generated file opened and saved in Sketchpad 5 |
| Graph Menu &gt; Mark Coordinate System | Pending |
| Graph Menu &gt; Grid Form | Partial: Square, Polar, Rectangular, and trigonometric X-axis formats decode and save; polar rings/rays, independent rectangular spacing, and π labels render in native/SVG; original and generated files opened in Sketchpad 5, including an edited rectangular unit; broader axis formatting pending |
| Graph Menu &gt; Show Grid | Partial: coordinate-system visibility decoded and saved; hidden frame components omitted from native/SVG rendering; original and generated hidden frames reopened in Sketchpad 5 |
| Graph Menu &gt; Dotted Grid | Partial: original grid flag decoded, native and SVG dots rendered, undo/redo and save/reopen tested; generated Cartesian and Polar dotted frames opened and saved byte-identically in Sketchpad 5 |
| Graph Menu &gt; Snap Points | Partial: page flag decoded and saved; free-point creation and drag snap to the nearest Cartesian or Polar grid intersection; original drag results matched numerically, and port-generated file opened, snapped again, and saved in Sketchpad 5; other point kinds pending |
| Graph Menu &gt; Plot Value on Axis/Path | Partial: selected t₁ parameter plots as a live code 95 point, while the value dialog plots a fixed numeric code 123 point; X/Y axes, segments, rays, lines, and circles decode, render, and save; original and generated cases opened and saved in Sketchpad 5; expressions, other numeric sources, and further paths pending |
| Graph Menu &gt; Plot Points/Plot as (x, y)/Plot as (r, θ) | Partial: original and generated one-column t₁ table points, including two-row tables, decode, render, follow frame drags, and save/reopen in Rust; integrated Cartesian and polar files opened and saved byte-identically in Sketchpad 5; arbitrary numeric sources and multiple plotted points pending |
| Graph Menu &gt; Plot Table Data | Partial: one- or two-row Cartesian plot from the verified t₂/t₁ table; duplicate rows combine, distinct rows produce separate points, and changing t₂ later updates the live table row while plotted points stay fixed; fractional original and generated files cross-open; other columns, further rows, and polar mode pending |
| Graph Menu &gt; Plot Function/Plot New Function | Partial: Cartesian `x` and `x²` plots update after origin drag and cross-open; other expressions and ranges pending |
| Graph Menu &gt; Plot Parametric Curve | Pending |

## Action Buttons

| Command | Status |
| --- | --- |
| Hide/Show | Partial: original and generated buttons cross-opened, click updates target visibility, save/reopen verified |
| Animation | Pending |
| Movement | Pending |
| Presentation | Pending |
| Link | Pending |
| Scroll | Pending |

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
| Select and drag free points | Partial |
| Point tool | Partial |
| Segment tool | Partial |
| Circle tool | Partial |
| Text, image, table, iteration, plotting | Partial: literal text, point labels, a single PNG picture, and Cartesian `x`/`x²` plots cross-open; table and iteration pending |
| Import, export, print | Partial: single PNG import, SVG/PDF export, and Linux printing for supported visible geometry; other image formats and physical print verification pending |
| Multi-page documents | Partial: page navigation and read-only complex objects |
| Cross-open generated files in Windows Sketchpad 5 | Partial: nine-object file edited and saved in Sketchpad 5, then edited in Rust and reopened under Wine |
