//! Newline-delimited MCP stdio. Only this module writes to stdout.
use super::{ToolCall, tool_list};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

const VERSION: &str = "2025-11-25";
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "error":{"code":code,"message":message}})
}

#[derive(Default)]
struct Session {
    initialized: bool,
    ready: bool,
}

impl Session {
    fn respond(
        &mut self,
        message: Value,
        dispatch: &mut impl FnMut(ToolCall) -> Result<Value, String>,
    ) -> Option<Value> {
        let id = message.get("id").cloned();
        let method = message.get("method").and_then(Value::as_str);
        if !message.is_object()
            || message["jsonrpc"] != "2.0"
            || method.is_none()
            || id
                .as_ref()
                .is_some_and(|id| !id.is_string() && !id.is_i64() && !id.is_u64())
        {
            return Some(error(Value::Null, -32600, "Invalid JSON-RPC request"));
        }
        let method = method.expect("validated method");
        let Some(id) = id else {
            if method == "notifications/initialized" && self.initialized {
                self.ready = true;
            }
            // Notifications, including cancellation, never execute tools.
            return None;
        };
        let params = message.get("params").cloned().unwrap_or_else(|| json!({}));
        if !params.is_object() {
            return Some(error(id, -32602, "params must be an object"));
        }
        let result = match method {
            "initialize" => {
                if self.initialized {
                    return Some(error(id, -32600, "The session is already initialized"));
                }
                if !params["protocolVersion"].is_string()
                    || !params["capabilities"].is_object()
                    || !params["clientInfo"]["name"].is_string()
                    || !params["clientInfo"]["version"].is_string()
                {
                    return Some(error(
                        id,
                        -32602,
                        "initialize requires protocolVersion, capabilities, and clientInfo",
                    ));
                }
                self.initialized = true;
                json!({
                    "protocolVersion":VERSION, "capabilities":{"tools":{"listChanged":false}},
                    "serverInfo":{"name":"sketchpad-rs", "version":env!("CARGO_PKG_VERSION")},
                    "instructions":"Tools access the document in this server's editor window. Object IDs and page numbers start at 1. Edits remain unsaved. Save through the window. SVG exports do not capture the GPU or interface."
                })
            }
            "ping" => json!({}),
            _ if !self.ready => {
                return Some(error(
                    id,
                    -32600,
                    "Initialize the session before calling tools",
                ));
            }
            "tools/list" => tool_list(),
            "tools/call" => {
                let Some(name) = params["name"].as_str() else {
                    return Some(error(id, -32602, "tools/call requires a tool name"));
                };
                if !tool_list()["tools"]
                    .as_array()
                    .expect("tool definitions")
                    .iter()
                    .any(|tool| tool["name"] == name)
                {
                    return Some(error(id, -32602, "Unknown tool"));
                }
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let call = serde_json::from_value(json!({"name":name,"arguments":arguments}));
                let outcome = call
                    .map_err(|error| format!("Invalid tool arguments: {error}"))
                    .and_then(dispatch);
                match outcome {
                    Ok(value) => {
                        json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":false})
                    }
                    Err(message) => {
                        json!({"content":[{"type":"text","text":message}],"isError":true})
                    }
                }
            }
            _ => return Some(error(id, -32601, "Method not found")),
        };
        Some(json!({"jsonrpc":"2.0", "id":id,"result":result}))
    }
}

pub(super) fn serve(
    mut reader: impl BufRead,
    mut writer: impl Write,
    mut dispatch: impl FnMut(ToolCall) -> Result<Value, String>,
) -> io::Result<()> {
    let mut session = Session::default();
    loop {
        let mut line = Vec::new();
        // Bound allocation before parsing. Close on an oversized message.
        let size = std::io::Read::take(&mut reader, (MAX_MESSAGE_BYTES + 1) as u64)
            .read_until(b'\n', &mut line)?;
        if size == 0 {
            return Ok(());
        }
        if size > MAX_MESSAGE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MCP message exceeds 1 MiB",
            ));
        }
        let response = match serde_json::from_slice(&line) {
            Ok(message) => session.respond(message, &mut dispatch),
            Err(_) => Some(error(Value::Null, -32700, "Invalid JSON")),
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut writer, &response)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initialize() -> Value {
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":VERSION,"capabilities":{},"clientInfo":{"name":"test","version":"1"}}})
    }

    #[test]
    fn handshake_notifications_and_structured_results() {
        let requests = [
            initialize(),
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            json!({"jsonrpc":"2.0","id":"list","method":"tools/list"}),
            json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_state"}}),
            json!({"jsonrpc":"2.0","method":"tools/call","params":{"name":"undo"}}),
        ];
        let input = requests
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let mut output = Vec::new();
        let mut calls = 0;
        serve(input.as_bytes(), &mut output, |call| {
            assert!(matches!(call, ToolCall::GetState(_)));
            calls += 1;
            Ok(json!({"dirty":true}))
        })
        .unwrap();
        let messages: Vec<Value> = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["result"]["protocolVersion"], VERSION);
        assert_eq!(messages[1]["id"], "list");
        assert_eq!(messages[1]["result"]["tools"].as_array().unwrap().len(), 9);
        assert_eq!(messages[2]["result"]["structuredContent"]["dirty"], true);
        assert_eq!(calls, 1);
    }

    #[test]
    fn invalid_requests_do_not_dispatch_edits() {
        let mut session = Session::default();
        let mut dispatch = |_| panic!("Invalid request must not execute");
        let call = json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"undo"}});
        assert_eq!(
            session.respond(call, &mut dispatch).unwrap()["error"]["code"],
            -32600
        );
        session.respond(initialize(), &mut dispatch);
        session.respond(
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            &mut dispatch,
        );
        for arguments in [
            json!({"id":-1,"x":1,"y":2}),
            json!({"id":1,"x":1}),
            json!({"id":1,"x":1,"y":2,"surprise":true}),
        ] {
            let response = session.respond(json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"move_point","arguments":arguments}}), &mut dispatch).unwrap();
            assert_eq!(response["result"]["isError"], true);
        }
        let response = session
            .respond(
                json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"missing"}}),
                &mut dispatch,
            )
            .unwrap();
        assert_eq!(response["error"]["code"], -32602);
    }

    #[test]
    fn parse_errors_recover_and_oversized_messages_close() {
        let mut output = Vec::new();
        serve(
            b"{broken\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n".as_slice(),
            &mut output,
            |_| unreachable!(),
        )
        .unwrap();
        let messages: Vec<Value> = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(messages[0]["error"]["code"], -32700);
        assert_eq!(messages[1]["result"], json!({}));
        let oversized = vec![b' '; MAX_MESSAGE_BYTES + 1];
        assert!(serve(oversized.as_slice(), Vec::new(), |_| unreachable!()).is_err());
    }
}
