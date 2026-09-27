# Custom Tools desktop controls

The Rust app was exercised on an isolated Linux X11 display with Mesa software
rendering, using a disposable copy of
`embedded_tool_square_registered_native_verified.gsp`. This desktop test used
neither Wine nor a Wine prefix. The app was built from the settled custom-tool
click-input implementation.

The top Custom Tools menu lists “Square Given Two Vertices”. Its submenu shows
“2 input(s), in order”, Apply to Selected Inputs, and Choose Inputs by Clicking.
The same tool and commands are available through the toolbox dropdown. The
creation entry is visible in both menus. Its dialog was opened from the main
menu and cancelled, preserving the saved sketch. These captures retain the
actual menus and dialog:

- [Main menu and ordered input commands](custom_tools_desktop_menu.png)
- [Toolbox route with the selected input points](custom_tools_desktop_toolbox.png)
- [Create New Tool name dialog](custom_tools_desktop_create_dialog.png)

Choosing Inputs by Clicking and clicking blank canvas positions `(800,450)` and
`(900,450)` created Free input objects 2 and 3. With the fitted viewport, their
saved world coordinates are `(373,241.5)` and
`(406.3333333333333,241.5)`. The resulting yellow square appears above that
baseline, with its upper corners at y approximately `208.16666666666669` and
labels C and D. The runtime selected its seven visible outputs; its four hidden
construction helpers remained hidden, and the input points remained unselected.
The [actual clicked result](custom_tools_desktop_clicked.png) shows 14 scene
objects and the successful tool status.

Ctrl+S wrote the actual desktop output to the 4,452-byte retained fixture
`tests/fixtures/embedded_tool_square_desktop_clicked.gsp`. Reopening it yields
14 objects, `partial_view=false`, and polygon 14 with ordered vertices
`[2,3,10,8]`. Helpers 5, 6, 7, and 9 are hidden. The independent tool namespace
still has 13 prototype objects and ordered input declaration `[2,1]`.

For the alternate route, the Select tool clicked point 3, then Shift-clicked
point 2. Toolbox > Custom Tools > Square Given Two Vertices > Apply to Selected
Inputs created the reversed square below the same baseline, demonstrating that
selection order is retained. Its lower corners have y approximately
`274.8333333333333`, with C on the right and D on the left. The
[actual selected-input result](custom_tools_desktop_selected.png) shows 25 scene
objects and the second invocation's seven visible outputs selected.

A second Ctrl+S wrote the 5,252-byte retained fixture
`tests/fixtures/embedded_tool_square_desktop_selected.gsp`. It contains segment
15 with endpoints `[3,2]` and polygon 25 with ordered vertices `[3,2,21,19]`;
helpers 16, 17, 18, and 20 are hidden. Both actual saved files reopen and encode
byte-identically. The editor regression
`actual_desktop_tool_saves_preserve_ordered_geometry_and_hidden_helpers` checks
these retained desktop files, evaluated corner locations, input order, hidden
helpers, prototype declaration, and byte preservation.

This is direct Rust desktop evidence for menu visibility, both invocation
routes, selection order, rendering, and Ctrl+S output. Separate copies of both
actual desktop files also opened and saved in Native Sketchpad with successful
exits and exact byte comparisons. The inspected Native captures show the upper
square, and then both the upper and reversed lower square:
[clicked input result](custom_tools_desktop_clicked_native.png) and
[selected input result](custom_tools_desktop_selected_native.png).
The registered input fixture and API-authored runtime fixtures have their own
Native evidence in `tool_runtime_analysis.md`.
