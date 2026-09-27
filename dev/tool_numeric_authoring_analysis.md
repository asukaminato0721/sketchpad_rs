# Numeric custom-tool authoring

Native's retained `embedded_tool_scalar_created_native_verified.gsp` independently
establishes creation from selected scalar parameter `t=2` and its calculation
`t+1`. The registered definition has a code0/class11/subtype33 Numeric given,
then code48/class11/subtype33 calculation with source reference1. These abstract
categories differ from ordinary subtype79 parameter and subtype51 calculation.
The existing runtime's acceptance of length, degree, or area inputs does not
establish the given category Native chooses when creating from those objects.

The bounded authoring implementation accepts a selected Scalar or Degrees
Parameter, or SegmentLength, and one directly dependent ParameterCalculation
on the active page. Both must be
live and numerically defined. Other numeric categories, multi-input templates,
and unselected calculation sources are rejected before history or drawing
changes. A semantic `ParameterCalculationGroup::remap_sources` maps the source
to local1; only the independently observed result header category becomes33.
No opaque payload words are interpreted as object IDs. The canonical abstract
given is extracted verbatim from the retained Native creation into
`assets/native_numeric_tool_given.bin`; its name changes through LabelAux's
validated setter. Original drawing objects, references, caption records, and
selection remain intact. The Numeric section uses the independently observed local Script View styles:
Tahoma18Normal for the given (index0) and Tahoma16Normal for the output (index1).
These are separate from the unchanged drawing caption styles.

Creation uses one structural history entry. Ordinary numeric input values are
bound by the existing runtime and remain live dependencies. Source values are
not captured in the abstract template. Other typed or measured categories remain separate Native controls until their
record categories and selection boundaries are established.

## Independent selected-boundary controls

Native selected only SegmentLength6 and Calculation7 in the retained ordinary
measured calculator drawing, leaving both endpoints and the segment unselected.
It accepted "Measured plus one", emitted Numeric given33 named `m[1]` and
calculation result33, and omitted the geometry prerequisites. Native selected a
35° parameter3 and calculation5 likewise emitted generic33/33. Both creation
runs had positive Script View captures, then independent second reopen/save
runs returned0 and byte-identical4402B (measurement) and4386B (degree). Retained
sources are `embedded_tool_measurement_created_native_verified.gsp` and
`embedded_tool_degree_created_native_verified.gsp`; captures are
`dev/tool_numeric_authoring_{measurement,degree}_native.png`.

The regression checks active source preservation, single-entry creation undo,
restoring the registered definition through redo, pruning measured geometry
from the template, live segment-length updates, and degree source updates.
Source names and calculation IDs are remapped semantically. A selected numeric
object whose actual source is unselected is rejected atomically. Length
parameters, area measurements, numeric calculation givens, multi-input numeric
templates, and other result families remain explicitly unsupported for
creation in this slice; runtime binding support remains separate.

## Actual port writer and Native execution

With the final Script View fonts, actual API-created files were opened and saved
by Native, then compared independently byte for byte: scalar2958B,
measurement4434B, degree4398B. Actual API-invoked files likewise passed Native0
and exact comparison: scalar3368B and measured4610B. Their positive captures
show `n=7` with `n+1=8` in normal18-point output, and a doubled live segment
`AB=10 cm` with `AB+1=11`. The five exact writer goldens are asserted by the
creation tests. They are distinct from the independent Native-authored controls.

The initial fresh scalar window had incomplete repainting after opening. Opening
Native's Manage Tools dialog caused all original `t=2` and `t+1=3` captions to
reappear, along with a newly added `n=5`; no font or record repair was needed.
Choosing the port-authored “Scalar plus one” and clicking `n=5` then created
scene object4 with source reference3 and displayed `n+1=6`. Its Script View
positively showed the generic given `t` and result `t+1`. This actual Native
execution is separate from the API-invoked file roundtrip. The retained positive
capture is `dev/tool_numeric_authoring_native_invocation.png`.
The actual Native invocation's3380B file independently reopened/saved with
terminal0 and exact comparison. It is retained as
`embedded_tool_numeric_authored_native_invocation_verified.gsp`. A separate
regression verifies the four scene objects, the source3 reference of result4,
the preserved original source1/result2, exact unchanged serialization, and live
recalculation after editing source3. This completes both writer acceptance and
actual Native execution of the authored definition.
