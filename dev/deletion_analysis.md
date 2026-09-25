# Deleting saved plot objects

The object writer removes deleted `0x07d0`–`0x07d7` groups and remaps surviving
`0x07d2` references. Function and plotted-point documents previously rejected
all deletion. The writer can safely remove a trailing group when no surviving
object follows the first deleted object. It can also delete ordinary objects
after the last function or plotted object, because those earlier object IDs
remain unchanged. Clear now encodes and reparses a candidate document before
committing it, and checks every surviving object's kind and references. A
rejected edit leaves the source document and undo history intact.

`tests/clear_plot_objects_replay.txt` reproduces five cases. Deleting a saved
Cartesian function plot leaves its definition; deleting the definition also
removes its dependent plot. Deleting a saved parametric plot or Cartesian
plotted point leaves their source functions or coordinate frame. A saved
ordinary point after a function plot can be removed while a later point
survives. All five generated files reopened as full editable documents in
the port. Sketchpad 5 opened and saved all five byte-identically. The corresponding fixtures use the
`*_cleared_generated.gsp` names in `tests/fixtures`.

`tests/clear_preplot_replay.txt` covers an independent free point before a
surviving Cartesian function plot, parametric plot, plotted point, table and
live path point, or two-column table. It also deletes an independent text
caption before a function plot, and removes a parameter with its dependent
table and path point while keeping the unrelated frame. It also copies one-
and two-column tables. Deletion
remaps the object IDs in `0x07d2` references and in the
embedded table group, including the table's source parameter IDs. Copy/Paste
remaps those same IDs and offsets pasted parameter and table anchors by 20
units, as in Sketchpad 5. The seven deletion outputs and both copied-table
outputs reopened and saved byte-identically in Sketchpad 5. Their generated
fixtures are in `tests/fixtures`.

Deleting a different kind of object before a surviving plot remains guarded:
the writer has not established the meaning of every native record that could
contain an object ID.
