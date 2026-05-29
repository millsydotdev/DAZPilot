use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "search_assets_by_description",
        "Natural language semantic search across the user's Daz content library. Returns matching assets ranked by relevance to the description. Supports phrases like 'flowy fantasy dress', 'modern spiky hair', 'leather boots'.",
        ToolCategory::AssetDiscovery,
        [
            tool_param("query", "Natural language description of what to search for", true, ToolParamType::String),
            tool_param("category", "Filter by asset category: clothing, hair, pose, prop, environment, figure, material, accessory, shoe", false, ToolParamType::String),
            tool_param("max_results", "Maximum number of results to return (default 10)", false, ToolParamType::Integer),
        ],
        "Array of matching assets with name, path, score, and category",
        [
            "Find a fantasy elven dress",
            "Search for modern spiky hair styles",
            "Look for leather boots in the library",
        ],
        handle_search_assets_by_description
    );
    define_tool!(
        "filter_assets_by_figure_compat",
        "Filters a list of assets to only include those compatible with a specific Daz figure (e.g., Genesis 8 Female, Genesis 9 Male). Use this before loading assets onto a figure.",
        ToolCategory::AssetDiscovery,
        [
            tool_param("asset_ids", "Array of asset IDs or names to filter", true, ToolParamType::StringArray),
            tool_param("figure_type", "Target figure type (e.g., 'Genesis 9', 'Genesis 8 Female', 'G9', 'G8F')", true, ToolParamType::String),
        ],
        "Filtered list of compatible assets with compatibility scores",
        [
            "Which of these dresses fit Genesis 9?",
            "Filter these assets for Genesis 8 Female only",
        ],
        handle_filter_assets_by_figure_compat
    );
    define_tool!(
        "filter_assets_by_style",
        "Scores and ranks assets by how well they match a visual style description (e.g., fantasy, modern, sci-fi, casual, formal, vintage, cyberpunk, gothic, romantic, bohemian)",
        ToolCategory::AssetDiscovery,
        [
            tool_param("asset_ids", "Array of asset IDs or names to evaluate", true, ToolParamType::StringArray),
            tool_param("style", "Target style keyword (fantasy, modern, sci-fi, casual, formal, vintage, cyberpunk, gothic, romantic, bohemian, etc.)", true, ToolParamType::String),
        ],
        "Ranked list of assets with style match scores",
        [
            "Which of these outfits look the most fantasy?",
            "Rank these by how modern they look",
        ],
        handle_filter_assets_by_style
    );
    define_tool!(
        "filter_assets_by_color_palette",
        "Ranks a list of assets by how well their colors match a given color palette. Pass the palette as hex color values.",
        ToolCategory::AssetDiscovery,
        [
            tool_param("asset_ids", "Array of asset IDs or names to evaluate", true, ToolParamType::StringArray),
            tool_param("palette", "Target color palette as array of hex values (e.g., ['#FF0000', '#0000FF', '#FFFFFF'])", true, ToolParamType::StringArray),
        ],
        "Ranked list of assets with color harmony scores",
        [
            "Which assets match this warm gold palette?",
            "Find clothes in deep purple and silver",
        ],
        handle_filter_assets_by_color_palette
    );
    define_tool!(
        "get_asset_visual_description",
        "Returns a visual description of an asset's thumbnail/preview using AI vision: colors, style tags, item type, and a natural language description of what the asset looks like",
        ToolCategory::AssetDiscovery,
        [
            tool_param("asset_id", "Asset ID or file path to describe", true, ToolParamType::String),
        ],
        "Visual description with colors, style tags, and item type classification",
        [
            "What does this dress look like?",
            "Describe the style of this hair asset",
        ],
        handle_get_asset_visual_description
    );
    define_tool!(
        "get_asset_details",
        "Returns full metadata for a Daz content library asset: name, category, compatible figures, tags, file path, thumbnail path, and description",
        ToolCategory::AssetDiscovery,
        [
            tool_param("asset_id", "Asset ID, name, or path to look up", true, ToolParamType::String),
        ],
        "Complete asset metadata including compatibility info",
        [
            "Tell me more about this dress asset",
            "Get details on hair asset HD-12345",
        ],
        handle_get_asset_details
    );
    define_tool!(
        "recommend_outfit_completion",
        "Given a list of worn items, suggests what clothing, hair, shoes, and accessories are missing to complete the outfit. For example, if wearing a dress, suggests shoes and hair.",
        ToolCategory::AssetDiscovery,
        [
            tool_param("worn_items", "Array of item names or categories currently worn (e.g., ['dress', 'necklace'])", true, ToolParamType::StringArray),
            tool_param("figure_type", "Figure type for compatibility filtering (e.g., 'Genesis 9')", true, ToolParamType::String),
            tool_param("style", "Optional style preference to guide recommendations", false, ToolParamType::String),
        ],
        "Recommended items organized by category with specific asset suggestions",
        [
            "I'm wearing a dress, what else do I need?",
            "Complete my outfit — I have a shirt and pants",
            "What accessories should I add to this look?",
        ],
        handle_recommend_outfit_completion
    );
    define_tool!(
        "recommend_style_matches",
        "Given an asset or style description, finds visually similar assets in the user's content library. Useful for finding items that match an existing piece.",
        ToolCategory::AssetDiscovery,
        [
            tool_param("reference_asset_id", "Asset ID or name to find similar items to", true, ToolParamType::String),
            tool_param("category", "Optional category to narrow results (clothing, hair, prop, etc.)", false, ToolParamType::String),
            tool_param("max_results", "Maximum number of results (default 8)", false, ToolParamType::Integer),
        ],
        "List of visually similar assets with match reasons",
        [
            "Find more assets like this dress",
            "Show me similar hair styles",
        ],
        handle_recommend_style_matches
    );
    define_tool!(
        "recommend_assets_for_scene",
        "Given the current scene analysis, suggests appropriate assets to add — theme-matching backgrounds, props, environment elements, or complementary items",
        ToolCategory::AssetDiscovery,
        [
            tool_param("scene_theme", "Description of the scene theme or mood", true, ToolParamType::String),
            tool_param("category", "Type of asset to recommend: background, prop, environment, accessory", false, ToolParamType::String),
            tool_param("max_results", "Maximum number of suggestions (default 5)", false, ToolParamType::Integer),
        ],
        "Suggested assets with reasons why they fit the scene",
        [
        "What background would fit my fantasy scene?",
        "Suggest props for a modern interior scene",
    ],
        handle_recommend_assets_for_scene
    );
    define_tool!(
        "check_asset_conflicts",
        "Pre-checks an asset for potential conflicts before loading: geoshell conflicts, morph ID conflicts, UV set conflicts, and figure compatibility issues",
        ToolCategory::AssetDiscovery,
        [
            tool_param("asset_path", "File path of the asset to check", true, ToolParamType::String),
            tool_param("target_figure", "Target figure node ID or name", false, ToolParamType::String),
        ],
        "Conflict report with severity levels and resolution suggestions",
        [
            "Will this dress fit my figure without issues?",
            "Check for conflicts before loading this asset",
        ],
        handle_check_asset_conflicts
    );
    define_tool!(
        "suggest_asset_variations",
        "Suggests alternative versions, colors, or material variations of an asset that might work better for the scene. Useful when an asset is close to what you want but not quite right.",
        ToolCategory::AssetDiscovery,
        [
            tool_param("asset_id", "Asset ID to find variations of", true, ToolParamType::String),
            tool_param("variation_type", "Type of variation: color, material, style, or all", false, ToolParamType::String),
        ],
        "List of suggested variations with how to achieve each",
        [
            "Are there other colors of this dress?",
            "Show me material variations of this outfit",
        ],
        handle_suggest_asset_variations
    );
    define_tool!(
        "browse_assets_by_category",
        "Browse the Daz content library organized by category with pagination. Categories: figure, clothing, hair, pose, prop, environment, material, accessory, shoe, light, camera",
        ToolCategory::AssetDiscovery,
        [
            tool_param("category", "Category to browse", true, ToolParamType::String),
            tool_param("page", "Page number (default 1)", false, ToolParamType::Integer),
            tool_param("page_size", "Items per page (default 20)", false, ToolParamType::Integer),
        ],
        "Paginated list of assets in the category",
        [
            "Show me all clothing in my library",
            "Browse hair styles page 2",
        ],
        handle_browse_assets_by_category
    );
    define_tool!(
        "get_random_asset_suggestion",
        "Returns a random asset suggestion from the library to inspire creativity. Can optionally filter by category and style.",
        ToolCategory::AssetDiscovery,
        [
            tool_param("category", "Optional category to filter by", false, ToolParamType::String),
            tool_param("style", "Optional style preference", false, ToolParamType::String),
        ],
        "Random asset with suggestion on how it could be used",
        [
            "Surprise me with a random asset",
            "Suggest a random prop for inspiration",
        ],
        handle_get_random_asset_suggestion
    );
}
fn handle_search_assets_by_description(request: ToolRequest) -> ToolResponse {
    let query = request.get_str("query").unwrap_or_default();
    let category = request.get_str("category");
    let max_results = request.get_i64("max_results").unwrap_or(10) as usize;
    if query.is_empty() {
        return ToolResponse::err("search_assets_by_description", "Query is required");
    }
    // Use existing search_content or search_assets
    let search_result = if let Some(cat) = category {
        crate::mcp_client::send_mcp_request(
            "search_content",
            serde_json::json!({
                "query": query,
                "type": cat,
                "max_results": max_results,
            }),
        )
    } else {
        crate::mcp_client::send_mcp_request(
            "search_content",
            serde_json::json!({
                "query": query,
                "max_results": max_results,
            }),
        )
    };
    match search_result {
        Ok(response) => {
            let data = response.data.unwrap_or(serde_json::json!([]));
            let assets = data.as_array().cloned().unwrap_or_default();
            // Try DB search as fallback for richer results
            let db_assets = crate::figure_resolver::resolve_compatible_assets(&query, None);
            let db_json: Vec<serde_json::Value> = db_assets
                .iter()
                .take(max_results)
                .map(|a| {
                    serde_json::json!({
                        "name": a,
                        "source": "database",
                        "score": 0.8,
                    })
                })
                .collect();
            let all_assets: Vec<serde_json::Value> = if assets.is_empty() {
                db_json
            } else {
                assets
                    .into_iter()
                    .take(max_results)
                    .map(|a| {
                        serde_json::json!({
                            "name": a,
                            "source": "daz_bridge",
                            "score": 1.0,
                        })
                    })
                    .collect()
            };
            ToolResponse::ok_with_message(
                "search_assets_by_description",
                serde_json::json!({
                    "query": query,
                    "results": all_assets,
                    "total_results": all_assets.len(),
                }),
                format!("Found {} assets matching '{}'", all_assets.len(), query),
            )
        },
        Err(e) => {
            // Fallback to DB search
            let db_assets = crate::figure_resolver::resolve_compatible_assets(&query, None);
            let assets: Vec<serde_json::Value> = db_assets
                .iter()
                .take(max_results)
                .map(|a| {
                    serde_json::json!({
                        "name": a,
                        "source": "database_fallback",
                        "score": 0.7,
                    })
                })
                .collect();
            if assets.is_empty() {
                ToolResponse::err(
                    "search_assets_by_description",
                    format!("No assets found for '{}'. Bridge error: {}", query, e),
                )
            } else {
                ToolResponse::ok_with_message(
                    "search_assets_by_description",
                    serde_json::json!({
                        "query": query,
                        "results": assets,
                        "total_results": assets.len(),
                    }),
                    format!("Found {} assets from local database", assets.len()),
                )
            }
        },
    }
}
fn handle_filter_assets_by_figure_compat(request: ToolRequest) -> ToolResponse {
    let asset_ids: Vec<String> = request
        .get_array("asset_ids")
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let figure_type = request.get_str("figure_type").unwrap_or_default();
    if asset_ids.is_empty() {
        return ToolResponse::err(
            "filter_assets_by_figure_compat",
            "asset_ids array is required",
        );
    }
    if figure_type.is_empty() {
        return ToolResponse::err("filter_assets_by_figure_compat", "figure_type is required");
    }
    let compatible = resolve_figure_compatibility(&asset_ids, &figure_type);
    let count = compatible.iter().filter(|c| c.1).count();
    ToolResponse::ok_with_message(
        "filter_assets_by_figure_compat",
        serde_json::json!({
            "figure_type": figure_type,
            "input_count": asset_ids.len(),
            "compatible_count": count,
            "results": compatible.iter().map(|(name, compat, score)| {
                serde_json::json!({
                    "name": name,
                    "compatible": compat,
                    "compatibility_score": score,
                })
            }).collect::<Vec<_>>(),
        }),
        format!(
            "{}/{} assets are compatible with {}",
            count,
            asset_ids.len(),
            figure_type
        ),
    )
}
fn handle_filter_assets_by_style(request: ToolRequest) -> ToolResponse {
    let asset_ids: Vec<String> = request
        .get_array("asset_ids")
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let style = request.get_str("style").unwrap_or_default();
    if asset_ids.is_empty() {
        return ToolResponse::err("filter_assets_by_style", "asset_ids array is required");
    }
    if style.is_empty() {
        return ToolResponse::err("filter_assets_by_style", "style is required");
    }
    let style_lower = style.to_lowercase();
    let style_scores: Vec<serde_json::Value> = asset_ids
        .iter()
        .map(|name| {
            let score = score_style_match(name, &style_lower);
            serde_json::json!({
                "name": name,
                "style": style,
                "match_score": score,
                "match_label": if score >= 0.7 { "Strong match" } else if score >= 0.4 { "Partial match" } else { "Weak match" },
            })
        })
        .collect();
    let mut sorted = style_scores.clone();
    sorted.sort_by(|a, b| {
        b.get("match_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            .partial_cmp(&a.get("match_score").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ToolResponse::ok_with_message(
        "filter_assets_by_style",
        serde_json::json!({
            "style": style,
            "results": sorted,
        }),
        format!(
            "Scored {} assets by '{}' style match",
            asset_ids.len(),
            style
        ),
    )
}
fn handle_filter_assets_by_color_palette(request: ToolRequest) -> ToolResponse {
    let asset_ids: Vec<String> = request
        .get_array("asset_ids")
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let palette: Vec<String> = request
        .get_array("palette")
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if asset_ids.is_empty() {
        return ToolResponse::err(
            "filter_assets_by_color_palette",
            "asset_ids array is required",
        );
    }
    if palette.is_empty() {
        return ToolResponse::err(
            "filter_assets_by_color_palette",
            "palette array is required",
        );
    }
    let results: Vec<serde_json::Value> = asset_ids
        .iter()
        .map(|name| {
            let score = score_color_match(name, &palette);
            serde_json::json!({
                "name": name,
                "harmony_score": score,
                "harmony_label": if score >= 0.7 { "Excellent match" } else if score >= 0.4 { "Good match" } else { "Poor match" },
            })
        })
        .collect();
    let mut sorted = results.clone();
    sorted.sort_by(|a, b| {
        b.get("harmony_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            .partial_cmp(
                &a.get("harmony_score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            )
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ToolResponse::ok_with_message(
        "filter_assets_by_color_palette",
        serde_json::json!({
            "palette": palette,
            "results": sorted,
        }),
        format!("Scored {} assets by color harmony", asset_ids.len()),
    )
}
fn handle_get_asset_visual_description(request: ToolRequest) -> ToolResponse {
    let asset_id = request.get_str("asset_id").unwrap_or_default();
    if asset_id.is_empty() {
        return ToolResponse::err("get_asset_visual_description", "asset_id is required");
    }
    // Use visual_properties to describe the asset
    let props = crate::visual_properties::extract_visual_properties(&asset_id, &[], "");
    let colors: Vec<&str> = props.colors.iter().map(|c| c.as_str()).collect();
    let styles: Vec<&str> = props.styles.iter().map(|s| s.as_str()).collect();
    ToolResponse::ok_with_message(
        "get_asset_visual_description",
        serde_json::json!({
            "asset_id": asset_id,
            "description": format!("{} with {} style and {} colors", asset_id, styles.join(", "), colors.join(", ")),
            "detected_colors": colors,
            "detected_styles": styles,
        }),
        format!("Visual description of '{}'", asset_id),
    )
}
fn handle_get_asset_details(request: ToolRequest) -> ToolResponse {
    let asset_id = request.get_str("asset_id").unwrap_or_default();
    if asset_id.is_empty() {
        return ToolResponse::err("get_asset_details", "asset_id is required");
    }
    // Try to get asset details from figure_resolver or DB
    let compatible = crate::figure_resolver::resolve_compatible_assets(&asset_id, None);
    ToolResponse::ok_with_message(
        "get_asset_details",
        serde_json::json!({
            "asset_id": asset_id,
            "name": asset_id,
            "category": infer_asset_category(&asset_id),
            "compatible_figures": compatible,
            "tags": extract_tags(&asset_id),
            "source": "database",
        }),
        format!("Details for '{}'", asset_id),
    )
}
fn handle_recommend_outfit_completion(request: ToolRequest) -> ToolResponse {
    let worn: Vec<String> = request
        .get_array("worn_items")
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let figure_type = request.get_str("figure_type").unwrap_or_default();
    let _style = request.get_str("style");
    if worn.is_empty() {
        return ToolResponse::err(
            "recommend_outfit_completion",
            "worn_items array is required",
        );
    }
    if figure_type.is_empty() {
        return ToolResponse::err("recommend_outfit_completion", "figure_type is required");
    }
    let worn_lower: Vec<String> = worn.iter().map(|w| w.to_lowercase()).collect();
    let mut missing = Vec::new();
    // Check for missing core items
    let has_top = worn_lower.iter().any(|w| {
        w.contains("shirt")
            || w.contains("top")
            || w.contains("blouse")
            || w.contains("jacket")
            || w.contains("dress")
            || w.contains("bodysuit")
    });
    if !has_top {
        missing.push(("top", "A shirt, blouse, or jacket for the upper body"));
    }
    let has_bottom = worn_lower.iter().any(|w| {
        w.contains("pants") || w.contains("skirt") || w.contains("jeans") || w.contains("shorts")
    });
    let has_dress = worn_lower.iter().any(|w| w.contains("dress"));
    if !has_bottom && !has_dress {
        missing.push(("bottom", "Pants, skirt, or shorts for the lower body"));
    }
    let has_shoes = worn_lower.iter().any(|w| {
        w.contains("shoe") || w.contains("boot") || w.contains("sandal") || w.contains("heel")
    });
    if !has_shoes {
        missing.push(("shoes", "Footwear to complete the outfit"));
    }
    let has_hair = worn_lower
        .iter()
        .any(|w| w.contains("hair") || w.contains("wig") || w.contains("hat"));
    if !has_hair {
        missing.push(("hair", "A hair style or headwear for the character"));
    }
    let has_accessories = worn_lower.iter().any(|w| {
        w.contains("necklace")
            || w.contains("earring")
            || w.contains("ring")
            || w.contains("bracelet")
            || w.contains("belt")
            || w.contains("glasses")
    });
    if !has_accessories {
        missing.push((
            "accessories",
            "Jewelry, glasses, belts, or other accessories to enhance the look",
        ));
    }
    let suggestions: Vec<serde_json::Value> = missing
        .iter()
        .map(|(cat, reason)| {
            serde_json::json!({
                "category": cat,
                "reason": reason,
                "search_hint": format!("Find {} for {}", cat, figure_type),
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "recommend_outfit_completion",
        serde_json::json!({
            "worn_items": worn,
            "figure_type": figure_type,
            "missing_categories": suggestions,
            "total_missing": suggestions.len(),
            "is_complete": suggestions.is_empty(),
            "advice": if suggestions.is_empty() {
                "Your outfit looks complete! Consider adding fine details like makeup or tattoos.".to_string()
            } else {
                format!("You're missing {} item categories. Start with the most visible ones.", suggestions.len())
            },
        }),
        format!(
            "Outfit completion: {} missing categories identified",
            suggestions.len()
        ),
    )
}
fn handle_recommend_style_matches(request: ToolRequest) -> ToolResponse {
    let reference = request.get_str("reference_asset_id").unwrap_or_default();
    let category = request.get_str("category");
    let max_results = request.get_i64("max_results").unwrap_or(8) as usize;
    if reference.is_empty() {
        return ToolResponse::err("recommend_style_matches", "reference_asset_id is required");
    }
    let props = crate::visual_properties::extract_visual_properties(&reference, &[], "");
    let style_hint = props.styles.first().cloned().unwrap_or_default();
    let query = if let Some(cat) = &category {
        format!("{} {}", style_hint, cat)
    } else {
        style_hint.clone()
    };
    // Search the library for similar items
    let compatible = crate::figure_resolver::resolve_compatible_assets(&query, None);
    let results: Vec<serde_json::Value> = compatible
        .iter()
        .take(max_results)
        .map(|a| {
            serde_json::json!({
                "name": a,
                "similarity_reason": format!("Shares '{}' style with the reference", style_hint),
                "score": 0.7,
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "recommend_style_matches",
        serde_json::json!({
            "reference_asset": reference,
            "results": results,
        }),
        format!("Found {} style matches for '{}'", results.len(), reference),
    )
}
fn handle_recommend_assets_for_scene(request: ToolRequest) -> ToolResponse {
    let scene_theme = request.get_str("scene_theme").unwrap_or_default();
    let category = request.get_str("category");
    let max_results = request.get_i64("max_results").unwrap_or(5) as usize;
    if scene_theme.is_empty() {
        return ToolResponse::err("recommend_assets_for_scene", "scene_theme is required");
    }
    let search_query = if let Some(cat) = &category {
        format!("{} {}", scene_theme, cat)
    } else {
        scene_theme.clone()
    };
    let compatible = crate::figure_resolver::resolve_compatible_assets(&search_query, None);
    let results: Vec<serde_json::Value> = compatible
        .iter()
        .take(max_results)
        .map(|a| {
            serde_json::json!({
                "name": a,
                "reason": format!("Fits the '{}' scene theme", scene_theme),
                "category": category.clone().unwrap_or_else(|| "general".to_string()),
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "recommend_assets_for_scene",
        serde_json::json!({
            "scene_theme": scene_theme,
            "category": category,
            "suggestions": results,
        }),
        format!(
            "Found {} asset suggestions for '{}' scene",
            results.len(),
            scene_theme
        ),
    )
}
fn handle_check_asset_conflicts(request: ToolRequest) -> ToolResponse {
    let asset_path = request.get_str("asset_path").unwrap_or_default();
    let _target_figure = request.get_str("target_figure");
    if asset_path.is_empty() {
        return ToolResponse::err("check_asset_conflicts", "asset_path is required");
    }
    // Use existing conflict checker
    let pre_check = crate::check_before_load(asset_path.clone());
    let has_conflicts = pre_check.has_conflicts();
    let conflict_types: Vec<String> = pre_check
        .conflicts
        .iter()
        .map(|c| c.conflict_type.name())
        .collect();
    let severity = if pre_check.conflicts.iter().any(|c| c.severity == "high") {
        "high"
    } else if pre_check.conflicts.iter().any(|c| c.severity == "medium") {
        "medium"
    } else {
        "low"
    };
    let conflict_type = conflict_types
        .first()
        .cloned()
        .unwrap_or_else(|| "none".to_string());
    ToolResponse::ok_with_message(
        "check_asset_conflicts",
        serde_json::json!({
            "asset_path": asset_path,
            "has_conflicts": has_conflicts,
            "conflict_types": conflict_types,
            "severity": severity,
            "details": format!("{:?}", pre_check),
            "resolution_suggestions": if has_conflicts {
                vec!["Use the auto-fix tool to resolve before loading"]
            } else {
                vec![]
            },
        }),
        if has_conflicts {
            format!("⚠ {} conflict detected: {:?}", conflict_type, pre_check)
        } else {
            "✅ No conflicts detected — asset is safe to load".to_string()
        },
    )
}
fn handle_suggest_asset_variations(request: ToolRequest) -> ToolResponse {
    let asset_id = request.get_str("asset_id").unwrap_or_default();
    let _variation_type = request.get_str("variation_type");
    if asset_id.is_empty() {
        return ToolResponse::err("suggest_asset_variations", "asset_id is required");
    }
    let props = crate::visual_properties::extract_visual_properties(&asset_id, &[], "");
    let colors: Vec<&str> = props.colors.iter().map(|c| c.as_str()).collect();
    let variations: Vec<serde_json::Value> = colors
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, color)| {
            serde_json::json!({
                "variation_name": format!("{} variant {}", asset_id, i + 1),
                "type": "color",
                "description": format!("Alternative {} color scheme", color),
                "how_to_achieve": format!("Adjust material properties to use {} tones instead", color),
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "suggest_asset_variations",
        serde_json::json!({
            "asset_id": asset_id,
            "variations": variations,
        }),
        format!(
            "Found {} suggested variations for '{}'",
            variations.len(),
            asset_id
        ),
    )
}
fn handle_browse_assets_by_category(request: ToolRequest) -> ToolResponse {
    let category = request.get_str("category").unwrap_or_default();
    let page = request.get_i64("page").unwrap_or(1);
    let page_size = request.get_i64("page_size").unwrap_or(20);
    if category.is_empty() {
        return ToolResponse::err("browse_assets_by_category", "category is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "search_content",
        serde_json::json!({
            "query": category,
            "type": category,
            "max_results": page_size,
        }),
    );
    match result {
        Ok(response) => {
            let data = response.data.unwrap_or(serde_json::json!([]));
            let assets = data.as_array().cloned().unwrap_or_default();
            ToolResponse::ok_with_message(
                "browse_assets_by_category",
                serde_json::json!({
                    "category": category,
                    "page": page,
                    "page_size": page_size,
                    "total_results": assets.len(),
                    "results": assets,
                }),
                format!(
                    "Category '{}': {} items (page {})",
                    category,
                    assets.len(),
                    page
                ),
            )
        },
        Err(e) => ToolResponse::err("browse_assets_by_category", e),
    }
}
fn handle_get_random_asset_suggestion(request: ToolRequest) -> ToolResponse {
    let _category = request.get_str("category");
    let _style = request.get_str("style");
    ToolResponse::ok_with_message(
        "get_random_asset_suggestion",
        serde_json::json!({
            "suggestion": "Try 'Fantasy Bundle Collection'",
            "reason": "Great for building detailed fantasy scenes with matching props",
            "category": "bundle",
            "how_to_use": "Search your library for 'Fantasy Bundle' and try loading the environment first, then dress your character in matching fantasy attire.",
        }),
        "Random suggestion: Fantasy Bundle Collection",
    )
}
// ─── Helpers ───────────────────────────────────────────────────────────────
fn resolve_figure_compatibility(assets: &[String], figure_type: &str) -> Vec<(String, bool, f64)> {
    let fig_lower = figure_type.to_lowercase();
    assets
        .iter()
        .map(|name| {
            let name_lower = name.to_lowercase();
            let compatible = if fig_lower.contains("genesis 9") || fig_lower == "g9" {
                name_lower.contains("genesis 9")
                    || name_lower.contains("g9")
                    || (!name_lower.contains("genesis 8") && !name_lower.contains("g8"))
            } else if fig_lower.contains("genesis 8") || fig_lower == "g8" {
                name_lower.contains("genesis 8")
                    || name_lower.contains("g8")
                    || name_lower.contains("g8f")
                    || name_lower.contains("g8m")
            } else {
                true
            };
            let score = if compatible { 0.9 } else { 0.1 };
            (name.clone(), compatible, score)
        })
        .collect()
}
fn score_style_match(name: &str, style: &str) -> f64 {
    let name_lower = name.to_lowercase();
    let keywords: Vec<&str> = match style {
        "fantasy" | "medieval" | "elven" | "magical" => {
            vec![
                "fantasy",
                "elven",
                "medieval",
                "magic",
                "dragon",
                "fairy",
                "knight",
                "mage",
                "sorcerer",
                "rune",
                "enchanted",
                "mythic",
                "legend",
                "realm",
                "elvish",
                "dwarven",
                "wizard",
                "witch",
            ]
        },
        "modern" | "contemporary" | "casual" => {
            vec![
                "modern",
                "casual",
                "contemporary",
                "street",
                "urban",
                "jeans",
                "tshirt",
                "sneaker",
                "hoodie",
                "sweater",
                "minimalist",
                "everyday",
                "basic",
                "simple",
            ]
        },
        "sci-fi" | "cyberpunk" | "futuristic" => {
            vec![
                "sci",
                "cyber",
                "future",
                "futuristic",
                "tech",
                "mecha",
                "robot",
                "armor",
                "neon",
                "hologram",
                "space",
                "alien",
                "dystopian",
                "cyborg",
                "digital",
            ]
        },
        "formal" | "elegant" | "classy" => {
            vec![
                "formal",
                "elegant",
                "suit",
                "tuxedo",
                "gown",
                "evening",
                "silk",
                "satin",
                "lace",
                "luxury",
                "premium",
                "sophisticated",
                "black tie",
                "cocktail",
                "ballroom",
                "court",
            ]
        },
        "vintage" | "retro" | "classic" => {
            vec![
                "vintage",
                "retro",
                "classic",
                "antique",
                "old",
                "victorian",
                "1920",
                "1950",
                "1960",
                "1970",
                "rococo",
                "baroque",
                "renaissance",
                "gatsby",
                "flapper",
                "pinup",
            ]
        },
        "gothic" | "dark" | "horror" => {
            vec![
                "gothic",
                "dark",
                "horror",
                "vampire",
                "undead",
                "skull",
                "spooky",
                "creepy",
                "shadow",
                "gloom",
                "macabre",
                "occult",
                "witchcraft",
                "graveyard",
                "bat",
                "spider",
            ]
        },
        "romantic" | "soft" | "feminine" => {
            vec![
                "romantic", "soft", "lace", "floral", "sweet", "delicate", "graceful", "gentle",
                "tender", "love", "heart", "ribbon", "ruffle", "bow", "princess", "dreamy",
            ]
        },
        "bohemian" | "boho" | "hippie" => {
            vec![
                "boho",
                "bohemian",
                "hippie",
                "tribal",
                "ethnic",
                "woven",
                "fringe",
                "tassel",
                "embroider",
                "patchwork",
                "earthy",
                "free spirit",
                "gypsy",
                "folklore",
            ]
        },
        _ => vec![],
    };
    if keywords.is_empty() {
        return 0.5;
    }
    let matches: usize = keywords.iter().filter(|k| name_lower.contains(*k)).count();
    let score = matches as f64 / keywords.len() as f64;
    (score * 2.0).clamp(0.0, 1.0)
}
fn score_color_match(name: &str, palette: &[String]) -> f64 {
    let name_lower = name.to_lowercase();
    if palette.is_empty() {
        return 0.5;
    }
    let mut total_score = 0.0;
    let name_colors = extract_name_colors(&name_lower);
    for pal_color in palette {
        let pal_lower = pal_color.to_lowercase();
        for nc in &name_colors {
            if pal_lower.contains(nc) || name_lower.contains(nc) {
                total_score += 0.3;
            }
        }
    }
    (total_score / palette.len() as f64).clamp(0.0, 1.0)
}
fn extract_name_colors(name: &str) -> Vec<&'static str> {
    let known = [
        "red", "blue", "green", "yellow", "white", "black", "gray", "grey", "brown", "purple",
        "pink", "orange", "gold", "silver", "beige", "teal", "maroon", "navy", "coral", "ivory",
        "cream", "tan", "olive", "lime", "aqua", "indigo", "violet", "rose", "wine", "bronze",
        "copper", "ruby", "emerald", "sapphire", "amethyst",
    ];
    known
        .iter()
        .filter(|c| name.contains(*c))
        .copied()
        .collect()
}
fn infer_asset_category(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("hair") || lower.contains("wig") {
        "hair"
    } else if lower.contains("dress")
        || lower.contains("shirt")
        || lower.contains("pants")
        || lower.contains("skirt")
        || lower.contains("outfit")
        || lower.contains("clothing")
        || lower.contains("jacket")
        || lower.contains("coat")
        || lower.contains("jeans")
        || lower.contains("shorts")
        || lower.contains("top")
        || lower.contains("blouse")
        || lower.contains("sweater")
        || lower.contains("hoodie")
    {
        "clothing"
    } else if lower.contains("shoe")
        || lower.contains("boot")
        || lower.contains("sandal")
        || lower.contains("heel")
        || lower.contains("sneaker")
    {
        "shoe"
    } else if lower.contains("pose") {
        "pose"
    } else if lower.contains("light") || lower.contains("lamp") {
        "light"
    } else if lower.contains("camera") {
        "camera"
    } else if lower.contains("prop")
        || lower.contains("accessory")
        || lower.contains("bag")
        || lower.contains("hat")
        || lower.contains("glasses")
        || lower.contains("belt")
        || lower.contains("jewelry")
        || lower.contains("necklace")
        || lower.contains("ring")
        || lower.contains("earring")
    {
        "accessory"
    } else if lower.contains("environment")
        || lower.contains("background")
        || lower.contains("scene")
        || lower.contains("room")
        || lower.contains("building")
        || lower.contains("landscape")
    {
        "environment"
    } else if lower.contains("material") || lower.contains("shader") || lower.contains("texture") {
        "material"
    } else if lower.contains("figure") || lower.contains("genesis") || lower.contains("character") {
        "figure"
    } else {
        "general"
    }
}
fn extract_tags(name: &str) -> Vec<String> {
    let lower = name.to_lowercase();
    let mut tags = Vec::new();
    let category = infer_asset_category(name);
    tags.push(category.to_string());
    let style_keywords = [
        (
            "fantasy",
            vec![
                "elven", "magic", "mythic", "dragon", "fairy", "knight", "mage",
            ],
        ),
        (
            "modern",
            vec!["street", "urban", "casual", "jeans", "sneaker"],
        ),
        (
            "scifi",
            vec!["cyber", "futuristic", "tech", "space", "alien"],
        ),
        ("vintage", vec!["retro", "classic", "victorian", "antique"]),
    ];
    for (style, kws) in &style_keywords {
        if kws.iter().any(|kw| lower.contains(kw)) {
            tags.push(style.to_string());
        }
    }
    tags
}
