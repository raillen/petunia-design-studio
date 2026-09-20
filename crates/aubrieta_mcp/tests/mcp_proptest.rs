use aubrieta_mcp::{McpRequest, McpServer};
use proptest::prelude::*;
use serde_json::json;

proptest! {
    #[test]
    fn json_dispatch_never_panics_on_arbitrary_string(input in ".*") {
        let mut server = McpServer::new();
        let _ = server.dispatch_json(&input);
    }

    #[test]
    fn unknown_method_always_returns_method_not_found(method in "[a-z_]{3,15}") {
        if !["aubrieta.discover", "document.summary", "export.svg"].contains(&method.as_str()) {
            let mut server = McpServer::new();
            let req = McpRequest::new(1, method, json!({}));
            let resp = server.dispatch(req);
            prop_assert!(resp.error.is_some());
            prop_assert_eq!(resp.error.unwrap().code, aubrieta_mcp::METHOD_NOT_FOUND);
        }
    }
}
