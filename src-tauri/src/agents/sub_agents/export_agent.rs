use crate::agents::{AgentAction, AgentRequest, AgentResponse};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages = vec![];

    if input.contains("export")
        || input.contains("save")
        || input.contains("output")
        || input.contains("write")
    {
        let format = if input.contains("fbx") {
            "fbx"
        } else if input.contains("obj") {
            "obj"
        } else if input.contains("duf") || input.contains("daz") {
            "duf"
        } else if input.contains("usd") || input.contains("usda") || input.contains("usdz") {
            "usd"
        } else if input.contains("gltf") || input.contains("glb") {
            "glb"
        } else if input.contains("abc") || input.contains("alembic") {
            "abc"
        } else if input.contains("png") || input.contains("image") {
            "png"
        } else {
            "duf"
        };

        let path = format!("scene_output.{}", format);
        let include = if input.contains("selected") || input.contains("selection") {
            "selected"
        } else {
            "all"
        };

        actions.push(AgentAction {
            action_type: "export_scene".to_string(),
            command: "export_scene".to_string(),
            args: vec![path, include.to_string(), format.to_string()],
        });
        messages.push(format!("Exporting scene as {} format.", format));
    }

    if input.contains("batch") && (input.contains("export") || input.contains("render")) {
        actions.push(AgentAction {
            action_type: "batch_export".to_string(),
            command: "batch_export".to_string(),
            args: vec![],
        });
        messages.push("Starting batch export.".to_string());
    }

    if input.contains("render")
        && (input.contains("export") || input.contains("output") || input.contains("save"))
    {
        actions.push(AgentAction {
            action_type: "render_export".to_string(),
            command: "render_preview".to_string(),
            args: vec![],
        });
        messages.push("Rendering preview for export.".to_string());
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No export intents identified.".to_string()),
            actions: vec![],
            sub_results: vec![],
        };
    }

    AgentResponse {
        success: true,
        result: Some(messages.join(" ")),
        error: None,
        actions,
        sub_results: vec![],
    }
}
