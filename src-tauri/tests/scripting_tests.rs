use dazpilot_lib::ai_system;
use dazpilot_lib::mcp_client;

#[test]
fn test_session_summary_state_tracking() {
    // Verify default session state is non-empty
    let summary = ai_system::get_session_summary();
    assert!(
        !summary.is_empty(),
        "Initial session summary should be initialized and non-empty."
    );
}

#[test]
fn test_scripting_command_schema_registration() {
    // Retrieve list of command schemas to verify run_script is registered correctly
    let schemas = mcp_client::get_mcp_command_list();
    let run_script_schema = schemas.iter().find(|c| c.name == "run_script");

    assert!(
        run_script_schema.is_some(),
        "run_script command schema must be registered and advertised in COMMAND_SCHEMAS."
    );

    let schema = run_script_schema.unwrap();
    assert!(
        schema.parameters.contains(&"script".to_string()),
        "run_script schema should declare a 'script' parameter."
    );
    assert!(
        schema.parameters.contains(&"args".to_string()),
        "run_script schema should declare an 'args' parameter."
    );
}
