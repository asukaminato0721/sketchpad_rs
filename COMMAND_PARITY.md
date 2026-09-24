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
| Close | Pending |
| Document Options | Pending |
| Page Setup | Pending |
| Print Preview | Pending |
| Print | Pending |
| Quit | Pending |
| Sketchpad LessonLink... | Outside release target (legacy service) |
| Sketchpad Sketch Exchange | Outside release target (legacy service) |

## Edit

| Command | Status |
| --- | --- |
| Undo | Partial |
| Redo | Partial |
| Cut | Pending |
| Copy | Pending |
| Paste | Pending |
| Clear | Pending |
| Action Buttons | Pending |
| Select All | Partial |
| Select Parents | Pending |
| Select Children | Pending |
| Crop Picture to Polygon | Pending |
| Split/Merge | Pending |
| Edit Definition | Pending |
| Properties | Pending |
| Preferences | Pending |

## Display

| Command | Status |
| --- | --- |
| Point Style | Pending |
| Line Style | Pending |
| Color | Pending |
| Text | Pending |
| Hide Objects | Partial |
| Show All Hidden | Partial |
| Show Labels | Pending |
| Label Objects | Pending |
| Trace Objects | Pending |
| Erase Traces | Pending |
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
| Point on Object | Pending |
| Midpoint | Partial |
| Intersection | Pending |
| Segment | Partial |
| Ray | Partial |
| Line | Partial |
| Parallel Line | Partial |
| Perpendicular Line | Partial |
| Circle by Center+Point | Partial |
| Angle Bisector | Pending |
| Circle by Center+Radius | Pending |
| Arc on Circle | Pending |
| Arc through 3 Points | Pending |
| Interior | Pending |
| Locus | Pending |

## Transform

| Command | Status |
| --- | --- |
| Mark Center | Pending |
| Mark Mirror | Pending |
| Mark Angle | Pending |
| Mark Ratio | Pending |
| Mark Vector | Pending |
| Mark Distance | Pending |
| Translate | Pending |
| Rotate | Pending |
| Dilate | Pending |
| Reflect | Pending |
| Iterate | Pending |
| Define Custom Transform | Pending |
| Edit Custom Transforms | Pending |

## Measure

| Command | Status |
| --- | --- |
| Length | Pending |
| Distance | Pending |
| Perimeter | Pending |
| Circumference | Pending |
| Angle | Pending |
| Area | Pending |
| Arc Angle | Pending |
| Arc Length | Pending |
| Radius | Pending |
| Ratio | Pending |
| Value of Point | Pending |
| Coordinates | Pending |
| Abscissa (x) | Pending |
| Ordinate (y) | Pending |
| Coordinate Distance | Pending |
| Slope | Pending |
| Equation | Pending |

## Number

| Command | Status |
| --- | --- |
| Number Menu &gt; New Parameter | Pending |
| Number Menu &gt; Calculate | Pending |
| Number Menu &gt; Tabulate | Pending |
| Number Menu &gt; Add Table Data | Pending |
| Number Menu &gt; Remove Table Data | Pending |
| Number Menu &gt; New Function | Pending |
| Number Menu &gt; Define Derivative Function | Pending |
| Number Menu &gt; Define Function from Drawing or Picture | Pending |

## Graph

| Command | Status |
| --- | --- |
| Graph Menu &gt; Define Coordinate System | Pending |
| Graph Menu &gt; Mark Coordinate System | Pending |
| Graph Menu &gt; Grid Form | Pending |
| Graph Menu &gt; Show Grid | Pending |
| Graph Menu &gt; Dotted Grid | Pending |
| Graph Menu &gt; Snap Points | Pending |
| Graph Menu &gt; Plot Value on Axis | Pending |
| Graph Menu &gt; Plot Points/Plot as (x, y)/Plot as (r, θ) | Pending |
| Graph Menu &gt; Plot Table Data | Pending |
| Graph Menu &gt; Plot Function/Plot New Function | Pending |
| Graph Menu &gt; Plot Parametric Curve | Pending |

## Action Buttons

| Command | Status |
| --- | --- |
| Hide/Show | Pending |
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
| Text, image, table, iteration, plotting | Pending |
| Import, export, print | Pending |
| Multi-page documents | Partial: page navigation and read-only complex objects |
| Cross-open generated files in Windows Sketchpad 5 | Partial: nine-object demo opened under Wine |
