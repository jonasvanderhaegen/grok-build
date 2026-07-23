//! MCP tools/call with progress-token subscription.
//!
//! rmcp already stamps `_meta.progressToken` on every peer request. This module
//! pairs that token with a live [`ProgressDispatcher`] subscription so
//! `notifications/progress` from the server is delivered while the call is open
//! (Claude Code / skybox-style live status), instead of being dropped on the
//! floor by a no-op `ClientHandler::on_progress`.

use std::time::Duration;

use futures::{FutureExt, StreamExt};
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResult, ClientRequest, ProgressNotificationParam, Request,
        ServerResult,
    },
    service::{PeerRequestOptions, RoleClient, RunningService, ServiceError},
};
use tokio::sync::mpsc;
use xai_tool_runtime::ToolProgress;

use crate::servers::GrokClientHandler;

/// Format a progress notification for UI / stream consumers.
pub fn format_progress_message(p: &ProgressNotificationParam) -> String {
    match (&p.message, p.total) {
        (Some(msg), Some(total)) if total > 0.0 => {
            let pct = ((p.progress / total) * 100.0).clamp(0.0, 100.0);
            format!("{msg} ({pct:.0}%)")
        }
        (Some(msg), Some(total)) => format!("{msg} ({}/{})", p.progress, total),
        (Some(msg), None) => msg.clone(),
        (None, Some(total)) if total > 0.0 => {
            let pct = ((p.progress / total) * 100.0).clamp(0.0, 100.0);
            format!("{:.0}/{:.0} ({pct:.0}%)", p.progress, total)
        }
        (None, _) => format!("progress {:.0}", p.progress),
    }
}

/// Convert an MCP progress notification into a harness [`ToolProgress`].
pub fn tool_progress_from_notification(p: &ProgressNotificationParam) -> ToolProgress {
    ToolProgress::Custom {
        subkind: "mcp_progress".into(),
        payload: serde_json::json!({
            "message": p.message,
            "progress": p.progress,
            "total": p.total,
            "text": format_progress_message(p),
        }),
    }
}

/// Extract display text from a streamed [`ToolProgress`] produced by this module.
pub fn tool_progress_display_text(p: &ToolProgress) -> Option<String> {
    match p {
        ToolProgress::Text { text } => Some(text.clone()),
        ToolProgress::Custom { subkind, payload } if subkind == "mcp_progress" => payload
            .get("text")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .or_else(|| {
                payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned)
            }),
        _ => None,
    }
}

/// Call `tools/call` with a progress subscription.
///
/// Progress notifications are forwarded to `on_progress` as they arrive.
/// The peer already attaches a unique `progressToken` in `_meta`.
pub async fn call_tool_with_progress(
    service: &RunningService<RoleClient, GrokClientHandler>,
    params: CallToolRequestParams,
    mut on_progress: impl FnMut(ProgressNotificationParam),
) -> Result<CallToolResult, ServiceError> {
    let handle = service
        .send_cancellable_request(
            ClientRequest::CallToolRequest(Request::new(params)),
            PeerRequestOptions::no_options(),
        )
        .await?;

    let mut subscriber = service
        .service()
        .progress_dispatcher()
        .subscribe(handle.progress_token.clone())
        .await;

    let response_fut = handle.await_response();
    tokio::pin!(response_fut);

    let result = loop {
        tokio::select! {
            biased;
            maybe = subscriber.next() => {
                match maybe {
                    Some(notification) => on_progress(notification),
                    // Subscription ended early; wait only for the RPC response.
                    None => break response_fut.await?,
                }
            }
            response = &mut response_fut => {
                break response?;
            }
        }
    };

    // Drain only notifications already buffered when the response arrived.
    while let Some(Some(notification)) = subscriber.next().now_or_never() {
        on_progress(notification);
    }

    match result {
        ServerResult::CallToolResult(r) => Ok(r),
        _ => Err(ServiceError::UnexpectedResponse),
    }
}

/// Same as [`call_tool_with_progress`] but maps progress into [`ToolProgress`]
/// and respects an outer timeout (tool timeout budget).
pub async fn call_tool_with_progress_timeout(
    service: &RunningService<RoleClient, GrokClientHandler>,
    params: CallToolRequestParams,
    timeout: Duration,
    progress_tx: Option<mpsc::UnboundedSender<ToolProgress>>,
) -> Result<CallToolResult, ServiceError> {
    let call = call_tool_with_progress(service, params, |notification| {
        if let Some(tx) = progress_tx.as_ref() {
            let _ = tx.send(tool_progress_from_notification(&notification));
        }
    });
    match tokio::time::timeout(timeout, call).await {
        Ok(r) => r,
        Err(_) => Err(ServiceError::Timeout { timeout }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::{NumberOrString, ProgressToken};

    fn notification(
        progress: f64,
        total: Option<f64>,
        message: Option<&str>,
    ) -> ProgressNotificationParam {
        let mut p =
            ProgressNotificationParam::new(ProgressToken(NumberOrString::Number(1)), progress);
        if let Some(t) = total {
            p = p.with_total(t);
        }
        if let Some(m) = message {
            p = p.with_message(m);
        }
        p
    }

    #[test]
    fn format_progress_prefers_message_and_percent() {
        let p = notification(2.0, Some(4.0), Some("indexing"));
        assert_eq!(format_progress_message(&p), "indexing (50%)");
    }

    #[test]
    fn format_progress_message_only() {
        let p = notification(1.0, None, Some("waiting"));
        assert_eq!(format_progress_message(&p), "waiting");
    }

    #[test]
    fn format_progress_numeric_fallback() {
        let p = notification(3.0, Some(10.0), None);
        assert_eq!(format_progress_message(&p), "3/10 (30%)");
    }

    #[test]
    fn tool_progress_round_trip_display_text() {
        let p = notification(1.0, Some(2.0), Some("half"));
        let tp = tool_progress_from_notification(&p);
        assert_eq!(
            tool_progress_display_text(&tp).as_deref(),
            Some("half (50%)")
        );
    }
}
