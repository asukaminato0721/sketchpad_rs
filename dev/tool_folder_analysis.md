# Tool folder workflow

## Native evidence

The native Custom Tools flyout and UTF-16 resources in `GSP5Chs.exe` identify
separate current-document and tool-folder sections plus **Choose Tool Folder**
and **Forget Tool Folder** commands. The retained captures are
`dev/tool_folder_native_menu.png` and `dev/tool_folder_native_chooser.png`.
The chooser is a standard folder dialog. Native feedback strings distinguish
document counts from tool counts and say newly added documents become available
after the next launch, which is consistent with a cached catalog.

The binary and UI probes do not settle recursive discovery, native ordering,
restart persistence, or whether invalid documents are reported or ignored.
Those behaviors remain outside the port's claimed boundary.

## Port implementation

**Choose Tool Folder** opens the platform folder picker. Cancelling changes
nothing. A successful choice scans only regular, top-level `.gsp` files in path
order and builds a replacement catalog before installing it. Each source
document is parsed once and held as an immutable in-memory snapshot; unreadable
files are skipped and counted in the status message. Ordinary documents with no
embedded tools are harmless. Duplicate tool names remain distinct because menu
labels include the source file name and runtime identity uses document and tool
indices, never the display name.

The catalog is editor-session state, not drawing state. It survives New, Close,
and Open, creates no drawing undo checkpoint, and is not persisted across app
restarts. Reloading or replacing the folder cancels any active folder-tool input
binding.

Folder entries reuse the same two invocation modes as embedded tools: apply to
ordered selected inputs or choose inputs by clicking. Input declarations and
prototype safety are read from the immutable source document. Invocation calls
the engine's external-prototype transaction, which materializes live destination
geometry, validates save/reparse safety, and leaves the source document
unchanged. The tool definition itself is not copied into the destination.

**Forget Tool Folder** drops the catalog and cancels an active tool binding.
Already materialized geometry and all other document state remain unchanged.

One integrated editor regression creates a source document containing a segment
and midpoint tool, loads it through the folder catalog, invokes it into another
drawing, verifies the catalog survives document replacement, and verifies
Forget does not alter completed geometry.

## Remaining boundary

Recursive discovery, Native's exact grouping/order, automatic refresh,
per-file diagnostics UI, restart persistence, example-document generation, and
Native comparison of the post-invocation destination namespace remain pending.
