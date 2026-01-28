//! E2E Full MCP Workflow Tests (bd-275)
//!
//! Comprehensive tests for complete MCP workflows with trace logging.
//! Covers:
//! - Server startup -> connect -> initialize -> operate -> shutdown
//! - Multiple sequential clients
//! - Interleaved tool/resource/prompt operations
//! - Error recovery during workflows
//! - Server with various handler configurations
//! - Resource template listing
//! - Client info propagation

use std::collections::HashMap;

use fastmcp::testing::prelude::*;
use fastmcp::{
    McpContext, McpResult, PromptHandler, PromptMessage, Resource, ResourceContent, ResourceHandler,
    ResourceTemplate, Role, ToolHandler,
};
use fastmcp_protocol::{Prompt, PromptArgument, Tool, ToolAnnotations};
use serde_json::json;

// ============================================================================
// Shared handler implementations
// ============================================================================

struct EchoToolHandler;

impl ToolHandler for EchoToolHandler {
    fn definition(&self) -> Tool {
        Tool {
            name: "echo".to_string(),
            description: Some("Echoes back the input".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            }),
            output_schema: None,
            icon: None,
            version: Some("1.0.0".to_string()),
            tags: vec![],
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        }
    }

    fn call(&self, _ctx: &McpContext, arguments: serde_json::Value) -> McpResult<Vec<Content>> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        Ok(vec![Content::Text {
            text: message.to_string(),
        }])
    }
}

struct CounterToolHandler;

impl ToolHandler for CounterToolHandler {
    fn definition(&self) -> Tool {
        Tool {
            name: "counter".to_string(),
            description: Some("Returns the call count (not truly stateful, returns arg)".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "value": { "type": "integer" }
                },
                "required": ["value"]
            }),
            output_schema: None,
            icon: None,
            version: None,
            tags: vec![],
            annotations: None,
        }
    }

    fn call(&self, _ctx: &McpContext, arguments: serde_json::Value) -> McpResult<Vec<Content>> {
        let value = arguments.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
        Ok(vec![Content::Text {
            text: value.to_string(),
        }])
    }
}

struct FailOnDemandToolHandler;

impl ToolHandler for FailOnDemandToolHandler {
    fn definition(&self) -> Tool {
        Tool {
            name: "fail_on_demand".to_string(),
            description: Some("Fails if 'fail' argument is true".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "fail": { "type": "boolean" },
                    "message": { "type": "string" }
                },
                "required": ["fail"]
            }),
            output_schema: None,
            icon: None,
            version: None,
            tags: vec![],
            annotations: None,
        }
    }

    fn call(&self, _ctx: &McpContext, arguments: serde_json::Value) -> McpResult<Vec<Content>> {
        let should_fail = arguments
            .get("fail")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if should_fail {
            let msg = arguments
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Requested failure");
            return Err(McpError::tool_error(msg));
        }
        Ok(vec![Content::Text {
            text: "Success".to_string(),
        }])
    }
}

struct StatusResourceHandler;

impl ResourceHandler for StatusResourceHandler {
    fn definition(&self) -> Resource {
        Resource {
            uri: "app://status".to_string(),
            name: "Server Status".to_string(),
            description: Some("Current server status".to_string()),
            mime_type: Some("application/json".to_string()),
            icon: None,
            version: None,
            tags: vec!["status".to_string()],
        }
    }

    fn read(&self, _ctx: &McpContext) -> McpResult<Vec<ResourceContent>> {
        let status = json!({
            "status": "healthy",
            "uptime_seconds": 42
        });
        Ok(vec![ResourceContent {
            uri: "app://status".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(status.to_string()),
            blob: None,
        }])
    }
}

struct ReadmeResourceHandler;

impl ResourceHandler for ReadmeResourceHandler {
    fn definition(&self) -> Resource {
        Resource {
            uri: "file:///README.md".to_string(),
            name: "README".to_string(),
            description: Some("Project README file".to_string()),
            mime_type: Some("text/markdown".to_string()),
            icon: None,
            version: Some("1.0.0".to_string()),
            tags: vec!["docs".to_string()],
        }
    }

    fn read(&self, _ctx: &McpContext) -> McpResult<Vec<ResourceContent>> {
        Ok(vec![ResourceContent {
            uri: "file:///README.md".to_string(),
            mime_type: Some("text/markdown".to_string()),
            text: Some("# Test Project\n\nThis is a test project.".to_string()),
            blob: None,
        }])
    }
}

struct HelpPromptHandler;

impl PromptHandler for HelpPromptHandler {
    fn definition(&self) -> Prompt {
        Prompt {
            name: "help".to_string(),
            description: Some("Get help on a topic".to_string()),
            arguments: vec![PromptArgument {
                name: "topic".to_string(),
                description: Some("The topic to get help on".to_string()),
                required: true,
            }],
            icon: None,
            version: None,
            tags: vec![],
        }
    }

    fn get(
        &self,
        _ctx: &McpContext,
        arguments: HashMap<String, String>,
    ) -> McpResult<Vec<PromptMessage>> {
        let topic = arguments.get("topic").map_or("general", String::as_str);
        Ok(vec![PromptMessage {
            role: Role::User,
            content: Content::Text {
                text: format!("Help me understand: {topic}"),
            },
        }])
    }
}

struct NoArgsPromptHandler;

impl PromptHandler for NoArgsPromptHandler {
    fn definition(&self) -> Prompt {
        Prompt {
            name: "system_prompt".to_string(),
            description: Some("Default system prompt".to_string()),
            arguments: vec![],
            icon: None,
            version: None,
            tags: vec![],
        }
    }

    fn get(
        &self,
        _ctx: &McpContext,
        _arguments: HashMap<String, String>,
    ) -> McpResult<Vec<PromptMessage>> {
        Ok(vec![PromptMessage {
            role: Role::Assistant,
            content: Content::Text {
                text: "You are a helpful assistant.".to_string(),
            },
        }])
    }
}

// ============================================================================
// Helper: build full workflow server
// ============================================================================

fn setup_workflow_server() -> TestClient {
    let (builder, client_transport, server_transport) = TestServer::builder()
        .with_name("workflow-test-server")
        .with_version("2.0.0")
        .build_server_builder();

    let server = builder
        .tool(EchoToolHandler)
        .tool(CounterToolHandler)
        .tool(FailOnDemandToolHandler)
        .resource(StatusResourceHandler)
        .resource(ReadmeResourceHandler)
        .resource_template(ResourceTemplate {
            uri_template: "file:///{path}".to_string(),
            name: "File Path".to_string(),
            description: Some("Access files by path".to_string()),
            mime_type: None,
            icon: None,
            version: None,
            tags: vec![],
        })
        .prompt(HelpPromptHandler)
        .prompt(NoArgsPromptHandler)
        .build();

    std::thread::spawn(move || {
        server.run_transport(server_transport);
    });

    TestClient::new(client_transport)
}

// ============================================================================
// Full lifecycle workflow tests
// ============================================================================

#[test]
fn workflow_complete_lifecycle() {
    let mut client = setup_workflow_server();

    // Phase 1: Initialize
    let init = client.initialize().unwrap();
    assert_eq!(init.server_info.name, "workflow-test-server");
    assert_eq!(init.server_info.version, "2.0.0");
    assert!(init.capabilities.tools.is_some());
    assert!(init.capabilities.resources.is_some());
    assert!(init.capabilities.prompts.is_some());

    // Phase 2: Discover capabilities
    let tools = client.list_tools().unwrap();
    assert_eq!(tools.len(), 3);

    let resources = client.list_resources().unwrap();
    assert_eq!(resources.len(), 2);

    let templates = client.list_resource_templates().unwrap();
    assert_eq!(templates.len(), 1);
    assert!(templates[0].uri_template.contains("{path}"));

    let prompts = client.list_prompts().unwrap();
    assert_eq!(prompts.len(), 2);

    // Phase 3: Execute operations
    let echo_result = client
        .call_tool("echo", json!({"message": "workflow test"}))
        .unwrap();
    match &echo_result[0] {
        Content::Text { text } => assert_eq!(text, "workflow test"),
        other => panic!("Expected text, got: {other:?}"),
    }

    let status = client.read_resource("app://status").unwrap();
    let status_json: serde_json::Value =
        serde_json::from_str(status[0].text.as_ref().unwrap()).unwrap();
    assert_eq!(status_json["status"], "healthy");

    let mut args = HashMap::new();
    args.insert("topic".to_string(), "MCP protocol".to_string());
    let help = client.get_prompt("help", args).unwrap();
    assert!(help[0]
        .content
        .as_text()
        .map_or(false, |t| t.contains("MCP protocol")));

    // Phase 4: Close
    client.close();
}

#[test]
fn workflow_discover_then_operate() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    // First discover all available tools
    let tools = client.list_tools().unwrap();
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Then call each tool that we find
    for name in &tool_names {
        let result = match *name {
            "echo" => client.call_tool("echo", json!({"message": "test"})),
            "counter" => client.call_tool("counter", json!({"value": 1})),
            "fail_on_demand" => client.call_tool("fail_on_demand", json!({"fail": false})),
            _ => continue,
        };
        assert!(result.is_ok(), "Tool {name} failed: {result:?}");
    }

    // Discover resources and read each one
    let resources = client.list_resources().unwrap();
    for resource in &resources {
        let content = client.read_resource(&resource.uri).unwrap();
        assert!(!content.is_empty(), "Resource {} returned empty", resource.uri);
    }
}

// ============================================================================
// Error recovery tests
// ============================================================================

#[test]
fn workflow_error_recovery_continues_after_tool_error() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    // Successful call
    let result = client
        .call_tool("fail_on_demand", json!({"fail": false}))
        .unwrap();
    match &result[0] {
        Content::Text { text } => assert_eq!(text, "Success"),
        other => panic!("Expected text, got: {other:?}"),
    }

    // Failed call
    let err = client
        .call_tool("fail_on_demand", json!({"fail": true, "message": "boom"}))
        .unwrap_err();
    assert!(err.message.contains("boom") || err.message.contains("Requested failure"));

    // Should still work after the error
    let result = client
        .call_tool("echo", json!({"message": "still alive"}))
        .unwrap();
    match &result[0] {
        Content::Text { text } => assert_eq!(text, "still alive"),
        other => panic!("Expected text, got: {other:?}"),
    }
}

#[test]
fn workflow_error_recovery_alternating_success_failure() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    for i in 0..5 {
        let should_fail = i % 2 == 1;
        let result = client.call_tool(
            "fail_on_demand",
            json!({"fail": should_fail, "message": format!("iteration {i}")}),
        );

        if should_fail {
            assert!(result.is_err(), "Iteration {i} should have failed");
        } else {
            assert!(result.is_ok(), "Iteration {i} should have succeeded");
        }
    }

    // Final verification: server still responsive
    let tools = client.list_tools().unwrap();
    assert_eq!(tools.len(), 3);
}

#[test]
fn workflow_unknown_tool_doesnt_break_session() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    // Call a valid tool
    let result = client
        .call_tool("echo", json!({"message": "before"}))
        .unwrap();
    assert_eq!(result.len(), 1);

    // Call an unknown tool (should fail)
    let err = client.call_tool("nonexistent", json!({}));
    assert!(err.is_err());

    // Call a valid tool again (should still work)
    let result = client
        .call_tool("echo", json!({"message": "after"}))
        .unwrap();
    match &result[0] {
        Content::Text { text } => assert_eq!(text, "after"),
        other => panic!("Expected text, got: {other:?}"),
    }
}

#[test]
fn workflow_unknown_resource_doesnt_break_session() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    // Read a valid resource
    let content = client.read_resource("app://status").unwrap();
    assert!(!content.is_empty());

    // Try to read an unknown resource (should fail)
    let err = client.read_resource("app://nonexistent");
    assert!(err.is_err());

    // Read a valid resource again (should still work)
    let content = client.read_resource("file:///README.md").unwrap();
    assert!(content[0].text.as_ref().unwrap().contains("Test Project"));
}

// ============================================================================
// Multiple sequential clients
// ============================================================================

#[test]
fn workflow_sequential_clients_same_server() {
    // Each client gets its own server - test that server setup pattern works repeatedly
    for i in 0..3 {
        let mut client = setup_workflow_server();
        let init = client.initialize().unwrap();
        assert_eq!(init.server_info.name, "workflow-test-server");

        let result = client
            .call_tool("echo", json!({"message": format!("client-{i}")}))
            .unwrap();
        match &result[0] {
            Content::Text { text } => assert_eq!(text, &format!("client-{i}")),
            other => panic!("Expected text, got: {other:?}"),
        }

        client.close();
    }
}

#[test]
fn workflow_two_independent_servers() {
    // Server A: tools only
    let (builder_a, client_a_transport, server_a_transport) = TestServer::builder()
        .with_name("server-a")
        .build_server_builder();
    let server_a = builder_a.tool(EchoToolHandler).build();
    std::thread::spawn(move || server_a.run_transport(server_a_transport));

    // Server B: resources only
    let (builder_b, client_b_transport, server_b_transport) = TestServer::builder()
        .with_name("server-b")
        .build_server_builder();
    let server_b = builder_b.resource(StatusResourceHandler).build();
    std::thread::spawn(move || server_b.run_transport(server_b_transport));

    // Client A
    let mut client_a = TestClient::new(client_a_transport);
    let init_a = client_a.initialize().unwrap();
    assert_eq!(init_a.server_info.name, "server-a");
    assert!(init_a.capabilities.tools.is_some());
    assert!(init_a.capabilities.resources.is_none());

    // Client B
    let mut client_b = TestClient::new(client_b_transport);
    let init_b = client_b.initialize().unwrap();
    assert_eq!(init_b.server_info.name, "server-b");
    assert!(init_b.capabilities.tools.is_none());
    assert!(init_b.capabilities.resources.is_some());

    // Use both
    let echo = client_a
        .call_tool("echo", json!({"message": "from A"}))
        .unwrap();
    match &echo[0] {
        Content::Text { text } => assert_eq!(text, "from A"),
        other => panic!("Expected text, got: {other:?}"),
    }

    let status = client_b.read_resource("app://status").unwrap();
    assert!(!status.is_empty());
}

// ============================================================================
// Resource template tests
// ============================================================================

#[test]
fn workflow_list_resource_templates() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    let templates = client.list_resource_templates().unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].name, "File Path");
    assert!(templates[0].uri_template.contains("{path}"));
}

// ============================================================================
// No-args prompt test
// ============================================================================

#[test]
fn workflow_get_prompt_without_arguments() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    let messages = client.get_prompt("system_prompt", HashMap::new()).unwrap();
    assert_eq!(messages.len(), 1);
    assert!(matches!(messages[0].role, Role::Assistant));
    match &messages[0].content {
        Content::Text { text } => {
            assert!(text.contains("helpful assistant"));
        }
        other => panic!("Expected text, got: {other:?}"),
    }
}

// ============================================================================
// Heavy sequential operations
// ============================================================================

#[test]
fn workflow_many_sequential_tool_calls() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    // 20 sequential tool calls
    for i in 0..20 {
        let msg = format!("message-{i}");
        let result = client.call_tool("echo", json!({"message": msg})).unwrap();
        match &result[0] {
            Content::Text { text } => assert_eq!(text, &msg),
            other => panic!("Expected text, got: {other:?}"),
        }
    }
}

#[test]
fn workflow_interleaved_list_and_call() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    // Interleave list and call operations
    for _ in 0..5 {
        let tools = client.list_tools().unwrap();
        assert_eq!(tools.len(), 3);

        let result = client
            .call_tool("counter", json!({"value": 42}))
            .unwrap();
        match &result[0] {
            Content::Text { text } => assert_eq!(text, "42"),
            other => panic!("Expected text, got: {other:?}"),
        }

        let resources = client.list_resources().unwrap();
        assert_eq!(resources.len(), 2);

        let content = client.read_resource("app://status").unwrap();
        assert!(!content.is_empty());
    }
}

// ============================================================================
// Server info and capability verification
// ============================================================================

#[test]
fn workflow_server_name_and_version() {
    let (builder, client_transport, server_transport) = TestServer::builder()
        .with_name("custom-name")
        .with_version("9.8.7")
        .build_server_builder();

    let server = builder.tool(EchoToolHandler).build();
    std::thread::spawn(move || server.run_transport(server_transport));

    let mut client = TestClient::new(client_transport);
    let init = client.initialize().unwrap();

    assert_eq!(init.server_info.name, "custom-name");
    assert_eq!(init.server_info.version, "9.8.7");
}

#[test]
fn workflow_capabilities_match_handlers() {
    let (builder, client_transport, server_transport) = TestServer::builder()
        .build_server_builder();

    let server = builder
        .tool(EchoToolHandler)
        .resource(StatusResourceHandler)
        .build();
    std::thread::spawn(move || server.run_transport(server_transport));

    let mut client = TestClient::new(client_transport);
    let init = client.initialize().unwrap();

    // Has tools and resources, but NOT prompts
    assert!(init.capabilities.tools.is_some());
    assert!(init.capabilities.resources.is_some());
    assert!(init.capabilities.prompts.is_none());
}

// ============================================================================
// Client info tests
// ============================================================================

#[test]
fn workflow_custom_client_info_accepted() {
    let (builder, client_transport, server_transport) = TestServer::builder()
        .build_server_builder();

    let server = builder.tool(EchoToolHandler).build();
    std::thread::spawn(move || server.run_transport(server_transport));

    let mut client = TestClient::new(client_transport)
        .with_client_info("my-custom-client", "5.0.0");

    // Should initialize successfully with custom client info
    let init = client.initialize().unwrap();
    assert!(init.capabilities.tools.is_some());

    // And should work normally
    let result = client
        .call_tool("echo", json!({"message": "custom client"}))
        .unwrap();
    assert_eq!(result.len(), 1);
}

// ============================================================================
// Annotation verification
// ============================================================================

#[test]
fn workflow_tool_annotations_preserved() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    let tools = client.list_tools().unwrap();
    let echo = tools.iter().find(|t| t.name == "echo").unwrap();

    let annotations = echo.annotations.as_ref().unwrap();
    assert_eq!(annotations.read_only, Some(true));
    assert_eq!(annotations.idempotent, Some(true));
}

#[test]
fn workflow_tool_descriptions_preserved() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    let tools = client.list_tools().unwrap();
    let echo = tools.iter().find(|t| t.name == "echo").unwrap();
    assert_eq!(
        echo.description.as_deref(),
        Some("Echoes back the input")
    );
    assert_eq!(echo.version.as_deref(), Some("1.0.0"));
}

#[test]
fn workflow_resource_metadata_preserved() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    let resources = client.list_resources().unwrap();
    let readme = resources.iter().find(|r| r.name == "README").unwrap();
    assert_eq!(readme.mime_type.as_deref(), Some("text/markdown"));
    assert_eq!(
        readme.description.as_deref(),
        Some("Project README file")
    );
    assert_eq!(readme.version.as_deref(), Some("1.0.0"));
}

#[test]
fn workflow_prompt_arguments_preserved() {
    let mut client = setup_workflow_server();
    client.initialize().unwrap();

    let prompts = client.list_prompts().unwrap();
    let help = prompts.iter().find(|p| p.name == "help").unwrap();

    assert_eq!(help.arguments.len(), 1);
    assert_eq!(help.arguments[0].name, "topic");
    assert!(help.arguments[0].required);

    let system = prompts
        .iter()
        .find(|p| p.name == "system_prompt")
        .unwrap();
    assert!(system.arguments.is_empty());
}

// ============================================================================
// Content type helper for assertions
// ============================================================================

trait ContentExt {
    fn as_text(&self) -> Option<&str>;
}

impl ContentExt for Content {
    fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text { text } => Some(text),
            _ => None,
        }
    }
}
