//! Model Context Protocol (MCP) Server for Aubrieta Design (09.15 & 09.29).
//!
//! Exposes discovery, query, transactional mutation, and export tools for AI agents
//! strictly mediated across Command and History boundaries without direct storage manipulation.

pub mod protocol;
pub mod server;

pub use protocol::{
    McpError, McpRequest, McpResponse, INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST,
    METHOD_NOT_FOUND, MUTATION_FAILED, PARSE_ERROR, STALE_REVISION,
};
pub use server::McpServer;
