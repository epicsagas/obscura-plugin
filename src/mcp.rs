use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::process::Command;

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: Value,
    id: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
    id: Value,
}

const ALL_TOOLS: &[(&str, &str, &str)] = &[
    (
        "browse",
        "Fetch a web page using Obscura headless browser. Returns clean text, raw HTML, or extracted links.",
        r#"{"type":"object","properties":{"url":{"type":"string","description":"URL to fetch"},"mode":{"type":"string","enum":["text","html","links"],"default":"text","description":"Output format"},"stealth":{"type":"boolean","default":false,"description":"Use stealth mode for bot-protected sites"},"eval":{"type":"string","description":"JavaScript to evaluate on the page"},"selector":{"type":"string","description":"CSS selector to wait for before extraction"}},"required":["url"]}"#,
    ),
    (
        "scrape_structured",
        "Extract structured JSON data from a web page by evaluating a JavaScript expression.",
        r#"{"type":"object","properties":{"url":{"type":"string","description":"URL to scrape"},"eval":{"type":"string","description":"JavaScript expression that returns JSON (use JSON.stringify)"},"stealth":{"type":"boolean","default":false,"description":"Use stealth mode"}},"required":["url","eval"]}"#,
    ),
];

fn call_tool(name: &str, args: &Value) -> Value {
    match name {
        "browse" => {
            let url = args["url"].as_str().unwrap_or("");
            let mode = args["mode"].as_str().unwrap_or("text");
            let stealth = args["stealth"].as_bool().unwrap_or(false);
            let eval_js = args["eval"].as_str();
            let selector = args["selector"].as_str();

            let mut cmd = Command::new("obscura");
            cmd.args(["fetch", url, "--quiet", "--dump", mode]);
            if stealth {
                cmd.arg("--stealth");
            }
            if let Some(js) = eval_js {
                cmd.args(["--eval", js]);
            }
            if let Some(sel) = selector {
                cmd.args(["--selector", sel]);
            }

            match cmd.output() {
                Ok(output) if output.status.success() => {
                    let text = String::from_utf8_lossy(&output.stdout);
                    let truncated = if text.len() > 10000 {
                        format!("{}...\n[truncated at 10000 chars]", &text[..10000])
                    } else {
                        text.into_owned()
                    };
                    json!({ "content": [{ "type": "text", "text": truncated }] })
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    json!({ "content": [{ "type": "text", "text": format!("Error (exit {}): {}", output.status, stderr) }], "isError": true })
                }
                Err(e) => {
                    json!({ "content": [{ "type": "text", "text": format!("Failed to execute obscura: {}", e) }], "isError": true })
                }
            }
        }
        "scrape_structured" => {
            let url = args["url"].as_str().unwrap_or("");
            let eval_js = args["eval"].as_str().unwrap_or("document.title");
            let stealth = args["stealth"].as_bool().unwrap_or(false);

            let mut cmd = Command::new("obscura");
            cmd.args(["fetch", url, "--quiet", "--eval", eval_js]);
            if stealth {
                cmd.arg("--stealth");
            }

            match cmd.output() {
                Ok(output) if output.status.success() => {
                    let text = String::from_utf8_lossy(&output.stdout);
                    json!({ "content": [{ "type": "text", "text": text.trim() }] })
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    json!({ "content": [{ "type": "text", "text": format!("Error (exit {}): {}", output.status, stderr) }], "isError": true })
                }
                Err(e) => {
                    json!({ "content": [{ "type": "text", "text": format!("Failed to execute obscura: {}", e) }], "isError": true })
                }
            }
        }
        _ => {
            json!({ "content": [{ "type": "text", "text": format!("Unknown tool: {}", name) }], "isError": true })
        }
    }
}

fn handle_request(req: &JsonRpcRequest) -> Value {
    match req.method.as_str() {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "obscura-plugin", "version": env!("CARGO_PKG_VERSION") }
        }),
        "notifications/initialized" => Value::Null,
        "tools/list" => {
            let tools: Vec<Value> = ALL_TOOLS
                .iter()
                .map(|(name, desc, schema)| {
                    json!({
                        "name": name,
                        "description": desc,
                        "inputSchema": serde_json::from_str::<Value>(schema).unwrap()
                    })
                })
                .collect();
            json!({ "tools": tools })
        }
        "tools/call" => {
            let name = req.params["name"].as_str().unwrap_or("");
            let args = &req.params["arguments"];
            call_tool(name, args)
        }
        _ => {
            json!({ "error": { "code": -32601, "message": format!("Method not found: {}", req.method) } })
        }
    }
}

pub fn run_stdio_server() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) },
                    "id": null
                });
                let _ = writeln!(stdout, "{}", err_resp);
                let _ = stdout.flush();
                continue;
            }
        };

        let result = handle_request(&req);

        // Notifications (no id) don't get a response
        if req.id.is_none() {
            continue;
        }

        let response = if result.get("error").is_some() {
            JsonRpcResponse {
                jsonrpc: "2.0".into(),
                result: None,
                error: Some(result["error"].clone()),
                id: req.id.unwrap_or(Value::Null),
            }
        } else {
            JsonRpcResponse {
                jsonrpc: "2.0".into(),
                result: Some(result),
                error: None,
                id: req.id.unwrap_or(Value::Null),
            }
        };

        let _ = writeln!(stdout, "{}", serde_json::to_string(&response).unwrap());
        let _ = stdout.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_have_required_fields() {
        for (name, desc, schema) in ALL_TOOLS {
            assert!(!name.is_empty());
            assert!(!desc.is_empty());
            let parsed: Value = serde_json::from_str(schema).unwrap();
            assert_eq!(parsed["type"], "object");
            assert!(parsed["properties"].is_object());
            assert!(parsed["required"].is_array());
        }
    }
}
