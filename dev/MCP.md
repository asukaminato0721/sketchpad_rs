# MCP debugging

The editor provides an opt-in MCP server over stdio. The client starts the
editor process. That process opens a real window and serves tools for the
document in that window. GUI edits and MCP edits share the same document.
This mode does not attach to a process that started without `--mcp`.

## Setup

Build the desktop binary:

```sh
cargo build --bin sketchpad_rs
```

Configure a stdio server in your MCP client. Replace `/absolute/path/sketchpad_rs`
with the repository path. A client that uses `mcpServers` accepts this entry:

```json
{
  "mcpServers": {
    "sketchpad": {
      "command": "/absolute/path/sketchpad_rs/target/debug/sketchpad_rs",
      "args": ["--mcp", "--demo"]
    }
  }
}
```

Replace `--demo` with an absolute `.gsp` path to open a file. Omit `--demo` to
start with a blank document. You can open other documents through the window.
The connection then operates on the newly opened document.

The client must inherit a working desktop environment, including `DISPLAY` or
`WAYLAND_DISPLAY` and the application's runtime libraries. The Nix package also
accepts `--mcp`. Its installed binary can replace the debug binary above.

The server implements MCP version `2025-11-25`. It provides `initialize`,
`ping`, `tools/list`, and `tools/call`. It uses the standard
[stdio transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)
and [tool results](https://modelcontextprotocol.io/specification/2025-11-25/server/tools).
Clients must send `notifications/initialized` after initialization.
Logs go to stderr. Stdout contains only MCP messages.

## Tools

| Tool | Arguments | Result |
| --- | --- | --- |
| `get_state` | None | Path, UI language, dirty state, page, selection, tool, drag, viewport, animation status, and history sizes |
| `list_objects` | Optional `page`, `offset`, `limit` | Object summaries and `next_offset` |
| `inspect_object` | `id` | Definition, direct parents and children, point position, numeric value and unit, source offsets |
| `select_objects` | `ids` | Updated editor state |
| `create_point` | `x`, `y`, optional `label` | Created IDs, selected IDs, and object summaries |
| `construct` | `kind`, ordered `inputs` | Created IDs, selected IDs, and object summaries |
| `move_point` | `id`, `x`, `y` | Updated object inspection |
| `set_parameter` | `id`, `value` | Updated object inspection |
| `undo` | None | Updated editor state |
| `redo` | None | Updated editor state |
| `export_svg` | None | `mime_type` and `svg` text for the current canvas viewport |

Object IDs and page numbers start at 1. List offsets start at 0. The default
list limit is 100, and the maximum is 500. A missing page selects the active
page. Deleted objects are omitted. A selection can contain up to 256 unique IDs.
Selection order follows the supplied array. An empty array clears selection.

Point coordinates use document space. The positive y direction is downwards.
Grid snapping and path constraints apply. The returned position is the actual
position after those rules run. Parameter values use their native units.
Inspection values can be null when evaluation is undefined or the field does
not apply to the object. Source offsets are byte offsets in the source document.
They can be null for newly created objects. The definition string uses Rust's
debug representation and is not a stable interchange format. Definitions are
limited to 16 KiB. `definition_truncated` reports whether the preview is incomplete.

All tool operations run on the window thread. Point and parameter edits use
document history. Failed moves do not create a checkpoint or clear redo history.
Undo and redo also stop motion, presentations, and sound, as the Edit menu does.
Edits require objects on the active page. Use the window to change pages.
Finish a GUI drag or a pending document confirmation before an MCP edit.

Tools return structured JSON and the same JSON as text. Domain errors return
`isError: true`. Protocol errors use JSON-RPC error responses. The server processes
one request at a time. It does not cancel an operation after dispatch. Clients
should check the state before retrying a mutation after a connection failure.
An input message cannot exceed 1 MiB.

Edits remain unsaved until you save through the window. No tool saves or
overwrites a file. When stdin closes, an unchanged document closes with the
process. A changed document displays the existing unsaved-changes dialog.
A client that terminates the process can bypass that dialog.

SVG export becomes available after the first canvas frame. It uses the current
page, zoom, and viewport. It does not capture the window, menus, GPU output, or
painted trace history. Use desktop screenshots to investigate rendering faults.

## Construct objects

`create_point` creates a free point on the active page. An optional label can
contain up to 256 characters. Grid snapping applies. The result contains
`created_ids`, `selected_ids`, and `objects`.

`construct` accepts existing IDs from the active page. Inputs must be distinct.
Use this order:

| Kind | Inputs |
| --- | --- |
| `segment`, `line`, `ray`, `midpoint` | Two points |
| `circle` | Center, point on circle |
| `parallel`, `perpendicular` | Point, straight path |
| `circle_by_segment` | Center, radius segment |
| `polygon` | 3–256 vertices in boundary order |
| `intersection` | Two intersecting paths |
| `angle_bisector` | First point, vertex, last point |
| `three_point_arc` | Start, through point, end |
| `center_arc` | Center, start, end |
| `arc_on_circle` | Circle, start point, end point |
| `point_on_object` | Host, optional point to project |
| `circle_interior` | Circle |
| `sector_interior` | Arc |

Each call creates one undo step, including helper objects and labels. A midpoint
can create a helper segment. An intersection can create multiple points.
`created_ids` contains all additions; `selected_ids` identifies the selected
results. At most 256 objects can be created by one call.

The existing document writer checks the result before the tool accepts it.
Unsupported additions to imported documents return an error. A failed operation
restores the document, selection, dirty flag, and undo/redo history.

For a blank document, send these calls after initialization:

```json
{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"create_point","arguments":{"x":100,"y":100,"label":"A"}}}
{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"create_point","arguments":{"x":300,"y":200,"label":"B"}}}
{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"construct","arguments":{"kind":"segment","inputs":[1,2]}}}
```

Use the returned IDs when the document already contains objects.

## Example

After initialization, call `list_objects` to obtain IDs. Then send:

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"inspect_object","arguments":{"id":1}}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"move_point","arguments":{"id":1,"x":340,"y":240}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"undo","arguments":{}}}
```

## Verification

Run protocol and document tests:

```sh
cargo test -p sketchpad-editor editor::mcp --lib
```

Run the window test on an isolated X11 display:

```sh
cargo build --bin sketchpad_rs
xvfb-run -a -s '-screen 0 1280x800x24' python3 dev/mcp_smoke.py
```

The window test requires `Xvfb`, `xdotool`, and Python 3. It checks the handshake,
live point edits, construction, dependency updates, a GUI drag, shared undo history, SVG output,
invalid requests, and connection shutdown. It uses the built-in demo and does
not modify document files.
