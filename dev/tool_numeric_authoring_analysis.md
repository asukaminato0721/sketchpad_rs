# Numeric custom-tool authoring

Native's retained `embedded_tool_scalar_created_native_verified.gsp` independently
establishes creation from selected scalar parameter `t=2` and its calculation
`t+1`. The registered definition has a code0/class11/subtype33 Numeric given,
then code48/class11/subtype33 calculation with source reference1. These abstract
categories differ from ordinary subtype79 parameter and subtype51 calculation.
The existing runtime's acceptance of length, degree, or area inputs does not
establish the given category Native chooses when creating from those objects.

The bounded authoring implementation accepts a selected Scalar Parameter and
one directly dependent ParameterCalculation on the active page. Both must be
live and numerically defined. Other numeric categories, multi-input templates,
and unselected calculation sources are rejected before history or drawing
changes. A semantic `ParameterCalculationGroup::remap_sources` maps the source
to local1; only the independently observed result header category becomes33.
No opaque payload words are interpreted as object IDs. The canonical abstract
given is extracted verbatim from the retained Native creation into
`assets/native_numeric_tool_given.bin`; its name changes through LabelAux's
validated setter. Original drawing objects, references, caption records, and
selection remain intact. The section uses the original drawing's font table.

Creation uses one structural history entry. Ordinary numeric input values are
bound by the existing runtime and remain live dependencies. Source values are
not captured in the abstract template. Typed or measured creation remains a
separate Native control until its record category and selection boundary are
established.
