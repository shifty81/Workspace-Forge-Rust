//! AI broker trait and stub implementation for NovaForge Workspace.
//!
//! Defines [`WorkspaceAI`] — a simple async query interface — and [`StubAI`],
//! a no-op implementation used until a real provider is configured.
//!
//! # Swapping in a real provider
//!
//! Implement [`WorkspaceAI`] for your provider struct and pass a
//! `Box<dyn WorkspaceAI>` to the AI Tool panel.  No UI code needs to change.

use std::future::Future;
use std::pin::Pin;

/// A pinned, heap-allocated, send-safe future yielding `T`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The AI broker interface used by all editor panels.
///
/// All methods are object-safe.  The default implementation returns immediately
/// (see [`StubAI`]).
pub trait WorkspaceAI: Send + Sync {
    /// Send a prompt and receive a response string asynchronously.
    fn query<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, String>;

    /// Returns `true` if the AI provider is reachable and configured.
    fn is_available(&self) -> bool;

    /// Short human-readable name of this provider (shown in the status bar).
    fn provider_name(&self) -> &str {
        "Stub"
    }
}

/// A no-op [`WorkspaceAI`] implementation used until a real provider is wired
/// in.  All queries return `"AI is not configured."` immediately.
pub struct StubAI;

impl WorkspaceAI for StubAI {
    fn query<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, String> {
        Box::pin(async { "AI is not configured.".to_string() })
    }

    fn is_available(&self) -> bool {
        false
    }

    fn provider_name(&self) -> &str {
        "Offline (stub)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_ai_is_not_available() {
        let ai = StubAI;
        assert!(!ai.is_available());
        assert_eq!(ai.provider_name(), "Offline (stub)");
    }

    #[test]
    fn stub_ai_query_returns_not_configured() {
        let ai = StubAI;
        // Block on the future using a simple inline executor.
        let result = futures::executor::block_on(ai.query("hello"));
        assert_eq!(result, "AI is not configured.");
    }
}
