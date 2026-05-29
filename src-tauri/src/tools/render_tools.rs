use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "set_render_settings",
        "Configures render engine settings: engine type, quality, resolution, samples, and denoising options.",
        ToolCategory::Rendering,
        [
            tool_param("engine", "Render engine: Iray, NVIDIA_Iray, or 3Delight (default Iray)", false, ToolParamType::String),
            tool_param("quality", "Render quality preset: draft, low, medium, high, ultra (default medium)", false, ToolParamType::String),
            tool_param("width", "Output width in pixels (default current viewport width)", false, ToolParamType::Integer),
            tool_param("height", "Output height in pixels (default current viewport height)", false, ToolParamType::Integer),
            tool_param("samples", "Max samples for Iray (default 2000)", false, ToolParamType::Integer),
            tool_param("denoise", "Enable denoising: true or false (default true)", false, ToolParamType::Boolean),
        ],
        "Result confirming render settings applied",
        [
            "Set render to ultra quality at 4K resolution",
            "Configure draft quality for quick previews",
            "Enable denoising with higher sample count",
        ],
        handle_set_render_settings
    );
    define_tool!(
        "render_preview",
        "Renders a preview image of the current viewport at the specified quality level.",
        ToolCategory::Rendering,
        [
            tool_param(
                "quality",
                "Preview quality: draft, low, medium (default draft)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "width",
                "Preview width in pixels (default 800)",
                false,
                ToolParamType::Integer
            ),
            tool_param(
                "height",
                "Preview height in pixels (default 600)",
                false,
                ToolParamType::Integer
            ),
        ],
        "Result with render status and image path",
        [
            "Render a quick draft preview",
            "Render a medium quality preview at 1080p",
        ],
        handle_render_preview
    );
    define_tool!(
        "configure_render_output",
        "Sets the file format, path, and naming convention for the final render output.",
        ToolCategory::Rendering,
        [
            tool_param(
                "format",
                "Output: png, jpg, tif, exr, bmp (default png)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "path",
                "Output directory path",
                false,
                ToolParamType::String
            ),
            tool_param(
                "filename",
                "Custom filename without extension",
                false,
                ToolParamType::String
            ),
            tool_param(
                "include_alpha",
                "Include alpha channel (default false)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result confirming output configuration",
        [
            "Save the render as PNG with alpha",
            "Configure EXR output to renders folder",
        ],
        handle_configure_render_output
    );
    define_tool!(
        "set_render_engine",
        "Selects the active render engine. Supports Iray, 3Delight, and GPU configuration.",
        ToolCategory::Rendering,
        [
            tool_param(
                "engine",
                "Engine: Iray, NVIDIA_Iray, 3Delight",
                true,
                ToolParamType::String
            ),
            tool_param(
                "use_gpu",
                "Use GPU rendering (default true)",
                false,
                ToolParamType::Boolean
            ),
            tool_param(
                "gpu_device_ids",
                "Comma-separated GPU device IDs",
                false,
                ToolParamType::String
            ),
        ],
        "Result confirming engine selection",
        [
            "Switch to 3Delight render engine",
            "Use NVIDIA Iray with GPU acceleration",
        ],
        handle_set_render_engine
    );
    define_tool!(
        "set_render_resolution",
        "Sets the exact render output resolution.",
        ToolCategory::Rendering,
        [
            tool_param("width", "Width in pixels", true, ToolParamType::Integer),
            tool_param("height", "Height in pixels", true, ToolParamType::Integer),
        ],
        "Result confirming resolution",
        ["Set render resolution to 4K", "Change to 1920x1080",],
        handle_set_render_resolution
    );
    define_tool!(
        "render_region",
        "Renders a specific rectangular region of the viewport for quick material/lighting tests.",
        ToolCategory::Rendering,
        [
            tool_param("x", "Region start X", true, ToolParamType::Integer),
            tool_param("y", "Region start Y", true, ToolParamType::Integer),
            tool_param("width", "Region width", true, ToolParamType::Integer),
            tool_param("height", "Region height", true, ToolParamType::Integer),
            tool_param(
                "quality",
                "Quality: draft, low, medium (default draft)",
                false,
                ToolParamType::String
            ),
        ],
        "Result with region render data",
        [
            "Test render the area around the character's face",
            "Render a small region to check material updates",
        ],
        handle_render_region
    );
    define_tool!(
        "queue_render",
        "Adds the current scene to a render queue for batch processing.",
        ToolCategory::Rendering,
        [
            tool_param(
                "pass_name",
                "Name for this render pass",
                false,
                ToolParamType::String
            ),
            tool_param(
                "camera_name",
                "Camera to render from",
                false,
                ToolParamType::String
            ),
        ],
        "Result confirming render was queued",
        [
            "Queue a render pass with the active camera",
            "Add a render pass from the overhead camera",
        ],
        handle_queue_render
    );
    define_tool!(
        "cancel_render",
        "Cancels the currently running render.",
        ToolCategory::Rendering,
        [tool_param(
            "clear_queue",
            "Clear pending renders (default false)",
            false,
            ToolParamType::Boolean
        ),],
        "Result confirming cancellation",
        ["Cancel the current render",],
        handle_cancel_render
    );
    define_tool!(
        "set_denoising",
        "Configures denoising settings for the active render engine.",
        ToolCategory::Rendering,
        [
            tool_param(
                "enabled",
                "Enable denoising (default true)",
                false,
                ToolParamType::Boolean
            ),
            tool_param(
                "strength",
                "Strength: low, medium, high (default medium)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "mode",
                "Mode: interactive, final, both (default final)",
                false,
                ToolParamType::String
            ),
        ],
        "Result confirming denoising settings",
        ["Enable high quality denoising", "Set denoising to medium",],
        handle_set_denoising
    );
}
fn handle_set_render_settings(request: ToolRequest) -> ToolResponse {
    let engine = request
        .get_str("engine")
        .unwrap_or_else(|| "Iray".to_string());
    let quality = request
        .get_str("quality")
        .unwrap_or_else(|| "medium".to_string());
    let width = request.get_i64("width");
    let height = request.get_i64("height");
    let samples = request.get_i64("samples").unwrap_or(2000);
    let denoise = request.get_bool("denoise").unwrap_or(true);
    let result = crate::mcp_client::send_mcp_request(
        "set_render_settings",
        serde_json::json!({ "engine": engine, "quality": quality, "width": width, "height": height, "samples": samples, "denoise": denoise }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_render_settings",
            serde_json::json!({ "engine": engine, "quality": quality, "samples": samples, "denoise": denoise }),
            format!(
                "Render settings applied: {} engine, {} quality, {} samples",
                engine, quality, samples
            ),
        ),
        Err(e) => ToolResponse::err("set_render_settings", e),
    }
}
fn handle_render_preview(request: ToolRequest) -> ToolResponse {
    let quality = request
        .get_str("quality")
        .unwrap_or_else(|| "draft".to_string());
    let width = request.get_i64("width").unwrap_or(800);
    let height = request.get_i64("height").unwrap_or(600);
    let result = crate::mcp_client::send_mcp_request(
        "render",
        serde_json::json!({ "quality": quality, "width": width, "height": height, "mode": "preview" }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "render_preview",
            serde_json::json!({ "quality": quality, "resolution": [width, height], "result": r.data }),
            format!("{} preview render started ({}x{})", quality, width, height),
        ),
        Err(e) => ToolResponse::err("render_preview", e),
    }
}
fn handle_configure_render_output(request: ToolRequest) -> ToolResponse {
    let format = request
        .get_str("format")
        .unwrap_or_else(|| "png".to_string());
    let path = request.get_str("path");
    let filename = request.get_str("filename");
    let include_alpha = request.get_bool("include_alpha").unwrap_or(false);
    let result = crate::mcp_client::send_mcp_request(
        "set_render_output",
        serde_json::json!({ "format": format, "path": path, "filename": filename, "include_alpha": include_alpha }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "configure_render_output",
            serde_json::json!({ "format": format, "include_alpha": include_alpha }),
            format!(
                "Output configured: {}{}",
                format.to_uppercase(),
                if include_alpha { " with alpha" } else { "" }
            ),
        ),
        Err(e) => ToolResponse::err("configure_render_output", e),
    }
}
fn handle_set_render_engine(request: ToolRequest) -> ToolResponse {
    let engine = request.get_str("engine").unwrap_or_default();
    let use_gpu = request.get_bool("use_gpu").unwrap_or(true);
    let gpu_devices = request.get_str("gpu_device_ids");
    if engine.is_empty() {
        return ToolResponse::err("set_render_engine", "engine is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_render_engine",
        serde_json::json!({ "engine": engine, "use_gpu": use_gpu, "gpu_devices": gpu_devices }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_render_engine",
            serde_json::json!({ "engine": engine, "gpu_acceleration": use_gpu }),
            format!(
                "Render engine set to '{}'{}",
                engine,
                if use_gpu { " (GPU)" } else { "" }
            ),
        ),
        Err(e) => ToolResponse::err("set_render_engine", e),
    }
}
fn handle_set_render_resolution(request: ToolRequest) -> ToolResponse {
    let width = request.get_i64("width").unwrap_or(1920);
    let height = request.get_i64("height").unwrap_or(1080);
    let result = crate::mcp_client::send_mcp_request(
        "set_resolution",
        serde_json::json!({ "width": width, "height": height }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_render_resolution",
            serde_json::json!({ "width": width, "height": height }),
            format!("Resolution set to {}x{}", width, height),
        ),
        Err(e) => ToolResponse::err("set_render_resolution", e),
    }
}
fn handle_render_region(request: ToolRequest) -> ToolResponse {
    let x = request.get_i64("x").unwrap_or(0);
    let y = request.get_i64("y").unwrap_or(0);
    let width = request.get_i64("width").unwrap_or(256);
    let height = request.get_i64("height").unwrap_or(256);
    let quality = request
        .get_str("quality")
        .unwrap_or_else(|| "draft".to_string());
    let result = crate::mcp_client::send_mcp_request(
        "render_region",
        serde_json::json!({ "x": x, "y": y, "width": width, "height": height, "quality": quality }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "render_region",
            serde_json::json!({ "region": [x, y, width, height], "result": r.data }),
            format!("Region rendered ({},{}) {}x{}", x, y, width, height),
        ),
        Err(e) => ToolResponse::err("render_region", e),
    }
}
fn handle_queue_render(request: ToolRequest) -> ToolResponse {
    let pass_name = request
        .get_str("pass_name")
        .unwrap_or_else(|| "Render Pass 1".to_string());
    let camera_name = request.get_str("camera_name");
    let result = crate::mcp_client::send_mcp_request(
        "queue_render",
        serde_json::json!({ "pass_name": pass_name, "camera_name": camera_name }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "queue_render",
            serde_json::json!({ "pass_name": pass_name, "result": r.data }),
            format!("Queued render '{}'", pass_name),
        ),
        Err(e) => ToolResponse::err("queue_render", e),
    }
}
fn handle_cancel_render(request: ToolRequest) -> ToolResponse {
    let clear_queue = request.get_bool("clear_queue").unwrap_or(false);
    let result = crate::mcp_client::send_mcp_request(
        "cancel_render",
        serde_json::json!({ "clear_queue": clear_queue }),
    );
    match result {
        Ok(_) => ToolResponse::ok(
            "cancel_render",
            serde_json::json!({ "cancelled": true, "queue_cleared": clear_queue }),
        ),
        Err(e) => ToolResponse::err("cancel_render", e),
    }
}
fn handle_set_denoising(request: ToolRequest) -> ToolResponse {
    let enabled = request.get_bool("enabled").unwrap_or(true);
    let strength = request
        .get_str("strength")
        .unwrap_or_else(|| "medium".to_string());
    let mode = request
        .get_str("mode")
        .unwrap_or_else(|| "final".to_string());
    let result = crate::mcp_client::send_mcp_request(
        "set_denoising",
        serde_json::json!({ "enabled": enabled, "strength": strength, "mode": mode }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_denoising",
            serde_json::json!({ "enabled": enabled, "strength": strength }),
            format!(
                "Denoising {} ({}, {})",
                if enabled { "enabled" } else { "disabled" },
                strength,
                mode
            ),
        ),
        Err(e) => ToolResponse::err("set_denoising", e),
    }
}
