//! MCP protocol messages.
//!
//! Request and response types for all MCP methods.

use serde::{Deserialize, Serialize};

use crate::jsonrpc::RequestId;
use crate::types::{
    ClientCapabilities, ClientInfo, Content, Prompt, PromptMessage, Resource, ResourceContent,
    ResourceTemplate, ServerCapabilities, ServerInfo, Tool,
};

// ============================================================================
// Progress Token
// ============================================================================

/// Progress token used to correlate progress notifications with requests.
///
/// Per MCP spec, progress tokens can be either strings or integers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProgressToken {
    /// String progress token.
    String(String),
    /// Integer progress token.
    Number(i64),
}

impl From<String> for ProgressToken {
    fn from(s: String) -> Self {
        ProgressToken::String(s)
    }
}

impl From<&str> for ProgressToken {
    fn from(s: &str) -> Self {
        ProgressToken::String(s.to_owned())
    }
}

impl From<i64> for ProgressToken {
    fn from(n: i64) -> Self {
        ProgressToken::Number(n)
    }
}

impl std::fmt::Display for ProgressToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgressToken::String(s) => write!(f, "{s}"),
            ProgressToken::Number(n) => write!(f, "{n}"),
        }
    }
}

/// Request metadata containing optional progress token.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestMeta {
    /// Progress token for receiving progress notifications.
    #[serde(rename = "progressToken", skip_serializing_if = "Option::is_none")]
    pub progress_token: Option<ProgressToken>,
}

// ============================================================================
// Initialize
// ============================================================================

/// Initialize request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version requested.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client info.
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Initialize response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version accepted.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Server info.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
    /// Optional instructions for the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

// ============================================================================
// Tools
// ============================================================================

/// tools/list request params.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListToolsParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// tools/list response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// List of available tools.
    pub tools: Vec<Tool>,
    /// Next cursor for pagination.
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// tools/call request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    /// Tool name to call.
    pub name: String,
    /// Tool arguments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    /// Request metadata (progress token, etc.).
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}

/// tools/call response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Tool output content.
    pub content: Vec<Content>,
    /// Whether the tool call errored.
    #[serde(
        rename = "isError",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub is_error: bool,
}

// ============================================================================
// Resources
// ============================================================================

/// resources/list request params.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListResourcesParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// resources/list response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
    /// List of available resources.
    pub resources: Vec<Resource>,
    /// Next cursor for pagination.
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// resources/templates/list request params.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListResourceTemplatesParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// resources/templates/list response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourceTemplatesResult {
    /// List of resource templates.
    #[serde(rename = "resourceTemplates")]
    pub resource_templates: Vec<ResourceTemplate>,
}

/// resources/read request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceParams {
    /// Resource URI to read.
    pub uri: String,
    /// Request metadata (progress token, etc.).
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}

/// resources/read response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    /// Resource contents.
    pub contents: Vec<ResourceContent>,
}

/// resources/subscribe request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeResourceParams {
    /// Resource URI to subscribe to.
    pub uri: String,
}

/// resources/unsubscribe request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeResourceParams {
    /// Resource URI to unsubscribe from.
    pub uri: String,
}

// ============================================================================
// Prompts
// ============================================================================

/// prompts/list request params.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListPromptsParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// prompts/list response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResult {
    /// List of available prompts.
    pub prompts: Vec<Prompt>,
    /// Next cursor for pagination.
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// prompts/get request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptParams {
    /// Prompt name.
    pub name: String,
    /// Prompt arguments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<std::collections::HashMap<String, String>>,
    /// Request metadata (progress token, etc.).
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}

/// prompts/get response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    /// Optional prompt description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Prompt messages.
    pub messages: Vec<PromptMessage>,
}

// ============================================================================
// Logging
// ============================================================================

/// Log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Debug level.
    Debug,
    /// Info level.
    Info,
    /// Warning level.
    Warning,
    /// Error level.
    Error,
}

/// logging/setLevel request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLogLevelParams {
    /// The log level to set.
    pub level: LogLevel,
}

// ============================================================================
// Notifications
// ============================================================================

/// Cancelled notification params.
///
/// Sent by either party to request cancellation of an in-progress request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelledParams {
    /// The ID of the request to cancel.
    #[serde(rename = "requestId")]
    pub request_id: RequestId,
    /// Optional reason for cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Whether the sender wants to await cleanup completion.
    #[serde(rename = "awaitCleanup", skip_serializing_if = "Option::is_none")]
    pub await_cleanup: Option<bool>,
}

/// Progress notification params.
///
/// Sent from server to client to report progress on a long-running operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressParams {
    /// Progress token (from original request's `_meta.progressToken`).
    #[serde(rename = "progressToken")]
    pub progress_token: ProgressToken,
    /// Progress value (0.0 to 1.0, or absolute values for indeterminate progress).
    pub progress: f64,
    /// Total expected progress (optional, for determinate progress).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    /// Optional progress message describing current status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ProgressParams {
    /// Creates a new progress notification.
    #[must_use]
    pub fn new(token: impl Into<ProgressToken>, progress: f64) -> Self {
        Self {
            progress_token: token.into(),
            progress,
            total: None,
            message: None,
        }
    }

    /// Creates a progress notification with total (determinate progress).
    #[must_use]
    pub fn with_total(token: impl Into<ProgressToken>, progress: f64, total: f64) -> Self {
        Self {
            progress_token: token.into(),
            progress,
            total: Some(total),
            message: None,
        }
    }

    /// Adds a message to the progress notification.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Returns the progress as a fraction (0.0 to 1.0) if total is known.
    #[must_use]
    pub fn fraction(&self) -> Option<f64> {
        self.total
            .map(|t| if t > 0.0 { self.progress / t } else { 0.0 })
    }
}

/// Resource updated notification params.
///
/// Sent from server to client when a subscribed resource changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdatedNotificationParams {
    /// Updated resource URI.
    pub uri: String,
}

/// Log message notification params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessageParams {
    /// Log level.
    pub level: LogLevel,
    /// Logger name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
    /// Log message data.
    pub data: serde_json::Value,
}

// ============================================================================
// Background Tasks (Docket/SEP-1686)
// ============================================================================

use crate::types::{TaskId, TaskInfo, TaskResult, TaskStatus};

/// tasks/list request params.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListTasksParams {
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by task status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
}

/// tasks/list response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksResult {
    /// List of tasks.
    pub tasks: Vec<TaskInfo>,
    /// Next cursor for pagination.
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// tasks/get request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskParams {
    /// Task ID to retrieve.
    pub id: TaskId,
}

/// tasks/get response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskResult {
    /// Task information.
    pub task: TaskInfo,
    /// Task result (if completed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<TaskResult>,
}

/// tasks/cancel request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskParams {
    /// Task ID to cancel.
    pub id: TaskId,
    /// Reason for cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// tasks/cancel response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskResult {
    /// Whether the cancellation was successful.
    pub cancelled: bool,
    /// Updated task information.
    pub task: TaskInfo,
}

/// tasks/submit request params.
///
/// Used to submit a new background task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTaskParams {
    /// Task type identifier.
    #[serde(rename = "taskType")]
    pub task_type: String,
    /// Task parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// tasks/submit response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTaskResult {
    /// Created task information.
    pub task: TaskInfo,
}

/// Task status change notification params.
///
/// Sent from server to client when a task status changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusNotificationParams {
    /// Task ID.
    pub id: TaskId,
    /// New task status.
    pub status: TaskStatus,
    /// Progress (0.0 to 1.0, if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    /// Progress message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Error message (if failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Task result (if completed successfully).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<TaskResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_resource_templates_params_serialization() {
        let params = ListResourceTemplatesParams { cursor: None };
        let value = serde_json::to_value(&params).expect("serialize params");
        assert_eq!(value, serde_json::json!({}));

        let params = ListResourceTemplatesParams {
            cursor: Some("next".to_string()),
        };
        let value = serde_json::to_value(&params).expect("serialize params with cursor");
        assert_eq!(value, serde_json::json!({ "cursor": "next" }));
    }

    #[test]
    fn list_resource_templates_result_serialization() {
        let result = ListResourceTemplatesResult {
            resource_templates: vec![ResourceTemplate {
                uri_template: "resource://{id}".to_string(),
                name: "Resource Template".to_string(),
                description: Some("Template description".to_string()),
                mime_type: Some("text/plain".to_string()),
            }],
        };

        let value = serde_json::to_value(&result).expect("serialize result");
        let templates = value
            .get("resourceTemplates")
            .expect("resourceTemplates key");
        let template = templates.get(0).expect("first resource template");

        assert_eq!(template["uriTemplate"], "resource://{id}");
        assert_eq!(template["name"], "Resource Template");
        assert_eq!(template["description"], "Template description");
        assert_eq!(template["mimeType"], "text/plain");
    }

    #[test]
    fn resource_updated_notification_serialization() {
        let params = ResourceUpdatedNotificationParams {
            uri: "resource://test".to_string(),
        };
        let value = serde_json::to_value(&params).expect("serialize params");
        assert_eq!(value, serde_json::json!({ "uri": "resource://test" }));
    }

    #[test]
    fn subscribe_unsubscribe_resource_params_serialization() {
        let subscribe = SubscribeResourceParams {
            uri: "resource://alpha".to_string(),
        };
        let value = serde_json::to_value(&subscribe).expect("serialize subscribe params");
        assert_eq!(value, serde_json::json!({ "uri": "resource://alpha" }));

        let unsubscribe = UnsubscribeResourceParams {
            uri: "resource://alpha".to_string(),
        };
        let value = serde_json::to_value(&unsubscribe).expect("serialize unsubscribe params");
        assert_eq!(value, serde_json::json!({ "uri": "resource://alpha" }));
    }

    #[test]
    fn logging_params_serialization() {
        let set_level = SetLogLevelParams {
            level: LogLevel::Warning,
        };
        let value = serde_json::to_value(&set_level).expect("serialize setLevel");
        assert_eq!(value, serde_json::json!({ "level": "warning" }));

        let log_message = LogMessageParams {
            level: LogLevel::Info,
            logger: Some("fastmcp::server".to_string()),
            data: serde_json::Value::String("hello".to_string()),
        };
        let value = serde_json::to_value(&log_message).expect("serialize log message");
        assert_eq!(value["level"], "info");
        assert_eq!(value["logger"], "fastmcp::server");
        assert_eq!(value["data"], "hello");
    }

    #[test]
    fn list_tasks_params_serialization() {
        let params = ListTasksParams {
            cursor: None,
            status: None,
        };
        let value = serde_json::to_value(&params).expect("serialize list tasks params");
        assert_eq!(value, serde_json::json!({}));

        let params = ListTasksParams {
            cursor: Some("next".to_string()),
            status: Some(TaskStatus::Running),
        };
        let value = serde_json::to_value(&params).expect("serialize list tasks params");
        assert_eq!(
            value,
            serde_json::json!({"cursor": "next", "status": "running"})
        );
    }

    #[test]
    fn submit_task_params_serialization() {
        let params = SubmitTaskParams {
            task_type: "demo".to_string(),
            params: None,
        };
        let value = serde_json::to_value(&params).expect("serialize submit task params");
        assert_eq!(value, serde_json::json!({"taskType": "demo"}));

        let params = SubmitTaskParams {
            task_type: "demo".to_string(),
            params: Some(serde_json::json!({"payload": 1})),
        };
        let value = serde_json::to_value(&params).expect("serialize submit task params");
        assert_eq!(
            value,
            serde_json::json!({"taskType": "demo", "params": {"payload": 1}})
        );
    }

    #[test]
    fn task_status_notification_serialization() {
        let params = TaskStatusNotificationParams {
            id: TaskId::from_string("task-1"),
            status: TaskStatus::Running,
            progress: Some(0.5),
            message: Some("halfway".to_string()),
            error: None,
            result: None,
        };
        let value = serde_json::to_value(&params).expect("serialize task status notification");
        assert_eq!(
            value,
            serde_json::json!({
                "id": "task-1",
                "status": "running",
                "progress": 0.5,
                "message": "halfway"
            })
        );
    }
}
