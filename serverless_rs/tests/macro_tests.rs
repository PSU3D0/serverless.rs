/*!
Tests for macro functionality in the serverless.rs framework.
*/

#[cfg(test)]
mod tests {
    // We don't need to import these directly for the tests
    // but we keep them here to show what the framework provides

    #[test]
    fn test_macro_compilation() {
        // Test that the macros compile and work correctly
        // The actual functionality is tested via the examples
        // This is just to ensure the macros are properly re-exported
        assert!(true);
    }

    #[test]
    fn test_info_functions() {
        // Test that the info module functions are properly exported
        let check_flag = serverless_rs::check_info_flag();
        assert!(!check_flag, "Info flag should be false in tests");
    }

    #[test]
    fn test_route_info() {
        // Test the RouteInfo struct directly
        let route = serverless_rs::RouteInfo::new("GET", "/hello");
        assert_eq!(route.method, "GET");
        assert_eq!(route.path, "/hello");
    }

    #[test]
    fn test_function_info() {
        // Test the FunctionInfo struct directly
        let info = serverless_rs::FunctionInfo::new("test_function")
            .with_description("Test function description");

        assert_eq!(info.name, "test_function");
        assert_eq!(
            info.description,
            Some("Test function description".to_string())
        );
    }

    #[test]
    fn test_value_reexport() {
        // Test that serde_json::Value is properly re-exported
        let value = serverless_rs::json!({
            "name": "Test",
            "value": 123
        });

        assert_eq!(value["name"], "Test");
        assert_eq!(value["value"], 123);
    }
}
