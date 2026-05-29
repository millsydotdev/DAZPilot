use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "load_clothing",
        "Loads a clothing item from the content library and optionally fits it to a figure. Supports searching by name, category, or style.",
        ToolCategory::Clothing,
        [
            tool_param("clothing_name", "Name or search term for the clothing item", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID to fit the clothing to (omit to just load into scene)", false, ToolParamType::String),
            tool_param("fit_mode", "How to fit: auto, manual_dof, simulate (default auto)", false, ToolParamType::String),
        ],
        "Result with loaded clothing node ID and fit status",
        [
            "Load a fantasy dress and fit to Genesis 9",
            "Load jeans and a t-shirt for my character",
            "Search for and load a leather jacket",
        ],
        handle_load_clothing
    );
    define_tool!(
        "fit_clothing",
        "Fits an existing clothing item to a figure using Daz's auto-fit or dForce simulation. Useful for clothing loaded separately or from external sources.",
        ToolCategory::Clothing,
        [
            tool_param("clothing_id", "Node ID of the clothing item to fit", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID to fit the clothing to", true, ToolParamType::String),
            tool_param("fit_type", "Fit type: auto_fit, dforce_simulate, manual (default auto_fit)", false, ToolParamType::String),
            tool_param("clear_morphs", "Clear existing fit morphs before fitting (default true)", false, ToolParamType::Boolean),
        ],
        "Result with fit status and morphs applied",
        [
            "Fit the loaded dress to Genesis 9",
            "Auto-fit the jacket to my character",
        ],
        handle_fit_clothing
    );
    define_tool!(
        "remove_clothing",
        "Removes a clothing item from a figure or from the scene entirely.",
        ToolCategory::Clothing,
        [
            tool_param(
                "clothing_id",
                "Node ID of the clothing item to remove",
                true,
                ToolParamType::String
            ),
            tool_param(
                "remove_from_scene",
                "Also delete from scene (true) or just unfit from figure (false, default false)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result confirming removal",
        [
            "Remove the jacket from Genesis 9",
            "Delete the loaded dress from the scene",
        ],
        handle_remove_clothing
    );
    define_tool!(
        "list_worn_items",
        "Lists all clothing and accessory items currently worn by a figure, with their fit status and material info.",
        ToolCategory::Clothing,
        [
            tool_param("figure_id", "Figure node ID to inspect", true, ToolParamType::String),
        ],
        "Result with list of worn items and their properties",
        [
            "What is my character wearing?",
            "List all clothing on Genesis 9",
        ],
        handle_list_worn_items
    );
    define_tool!(
        "set_clothing_parameters",
        "Adjusts clothing fit parameters such as length, tightness, and drape. Useful for fine-tuning the appearance of fitted clothing.",
        ToolCategory::Clothing,
        [
            tool_param("clothing_id", "Node ID of the clothing item", true, ToolParamType::String),
            tool_param("parameter", "Parameter to adjust: length, tightness, waist, bust, hips, inseam, custom", true, ToolParamType::String),
            tool_param("value", "Parameter value from -1.0 to 1.0 (negative = smaller/shorter, positive = larger/longer)", true, ToolParamType::Number),
        ],
        "Result confirming parameter change",
        [
            "Shorten the dress length by 20%",
            "Loosen the jacket tightness to 0.3",
            "Adjust the waist of the pants to -0.2",
        ],
        handle_set_clothing_parameters
    );
    define_tool!(
        "suggest_outfit_completion",
        "Given a figure's current outfit, suggests additional items that would complete the look (shoes, accessories, outerwear).",
        ToolCategory::Clothing,
        [
            tool_param("figure_id", "Figure node ID to analyze", true, ToolParamType::String),
            tool_param("style", "Style preference: casual, formal, fantasy, scifi, casual (default match existing)", false, ToolParamType::String),
        ],
        "Result with suggested clothing items and reasoning",
        [
            "What else should I add to complete this outfit?",
            "Suggest accessories for my fantasy character",
            "What shoes would go with this outfit?",
        ],
        handle_suggest_outfit_completion
    );
}
fn handle_load_clothing(request: ToolRequest) -> ToolResponse {
    let clothing_name = request.get_str("clothing_name").unwrap_or_default();
    let figure_id = request.get_str("figure_id");
    let fit_mode = request
        .get_str("fit_mode")
        .unwrap_or_else(|| "auto".to_string());
    if clothing_name.is_empty() {
        return ToolResponse::err("load_clothing", "clothing_name is required");
    }
    let mut params = serde_json::json!({
        "name": clothing_name,
        "fit_mode": fit_mode,
    });
    if let Some(fid) = figure_id {
        params["figure_id"] = serde_json::json!(fid);
    }
    let result = crate::mcp_client::send_mcp_request("load_clothing", params);
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "load_clothing",
            serde_json::json!({ "result": r.data }),
            format!("Loaded '{}'", clothing_name),
        ),
        Err(e) => ToolResponse::err("load_clothing", e),
    }
}
fn handle_fit_clothing(request: ToolRequest) -> ToolResponse {
    let clothing_id = request.get_str("clothing_id").unwrap_or_default();
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let fit_type = request
        .get_str("fit_type")
        .unwrap_or_else(|| "auto_fit".to_string());
    let clear_morphs = request.get_bool("clear_morphs").unwrap_or(true);
    if clothing_id.is_empty() || figure_id.is_empty() {
        return ToolResponse::err("fit_clothing", "clothing_id and figure_id are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "fit_clothing",
        serde_json::json!({
            "clothing_id": clothing_id,
            "figure_id": figure_id,
            "fit_type": fit_type,
            "clear_morphs": clear_morphs,
        }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "fit_clothing",
            serde_json::json!({ "result": r.data }),
            format!("Fitted '{}' to '{}'", clothing_id, figure_id),
        ),
        Err(e) => ToolResponse::err("fit_clothing", e),
    }
}
fn handle_remove_clothing(request: ToolRequest) -> ToolResponse {
    let clothing_id = request.get_str("clothing_id").unwrap_or_default();
    let remove_from_scene = request.get_bool("remove_from_scene").unwrap_or(false);
    if clothing_id.is_empty() {
        return ToolResponse::err("remove_clothing", "clothing_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "remove_clothing",
        serde_json::json!({ "clothing_id": clothing_id, "remove_from_scene": remove_from_scene }),
    );
    match result {
        Ok(_) => {
            let action = if remove_from_scene {
                "Deleted"
            } else {
                "Unfitted"
            };
            ToolResponse::ok_with_message(
                "remove_clothing",
                serde_json::json!({ "clothing_id": clothing_id }),
                format!("{} '{}'", action, clothing_id),
            )
        },
        Err(e) => ToolResponse::err("remove_clothing", e),
    }
}
fn handle_list_worn_items(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    if figure_id.is_empty() {
        return ToolResponse::err("list_worn_items", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "list_worn_items",
        serde_json::json!({ "figure_id": figure_id }),
    );
    match result {
        Ok(r) => {
            let items = r.data.unwrap_or(serde_json::json!([]));
            ToolResponse::ok_with_message(
                "list_worn_items",
                serde_json::json!({ "figure_id": figure_id, "items": items }),
                format!(
                    "Found {} items on '{}'",
                    items.as_array().map(|a| a.len()).unwrap_or(0),
                    figure_id
                ),
            )
        },
        Err(e) => ToolResponse::err("list_worn_items", e),
    }
}
fn handle_set_clothing_parameters(request: ToolRequest) -> ToolResponse {
    let clothing_id = request.get_str("clothing_id").unwrap_or_default();
    let parameter = request.get_str("parameter").unwrap_or_default();
    let value = request.get_f64("value").unwrap_or(0.0);
    if clothing_id.is_empty() || parameter.is_empty() {
        return ToolResponse::err(
            "set_clothing_parameters",
            "clothing_id and parameter are required",
        );
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_clothing_params",
        serde_json::json!({
            "clothing_id": clothing_id,
            "parameter": parameter,
            "value": value,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_clothing_parameters",
            serde_json::json!({ "clothing_id": clothing_id, "parameter": parameter, "value": value }),
            format!("Set '{}' = {:.2} on '{}'", parameter, value, clothing_id),
        ),
        Err(e) => ToolResponse::err("set_clothing_parameters", e),
    }
}
fn handle_suggest_outfit_completion(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let style = request.get_str("style");
    if figure_id.is_empty() {
        return ToolResponse::err("suggest_outfit_completion", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "suggest_outfit",
        serde_json::json!({ "figure_id": figure_id, "style": style }),
    );
    match result {
        Ok(r) => {
            let suggestions = r.data.unwrap_or(serde_json::json!([]));
            ToolResponse::ok_with_message(
                "suggest_outfit_completion",
                serde_json::json!({ "figure_id": figure_id, "suggestions": suggestions }),
                "Outfit suggestions generated",
            )
        },
        Err(e) => ToolResponse::err("suggest_outfit_completion", e),
    }
}
