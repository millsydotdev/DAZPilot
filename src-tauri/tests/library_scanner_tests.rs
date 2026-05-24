use dazpilot_lib::library_scanner;
use std::fs;

#[test]
fn test_recursive_library_scanner_pipeline() {
    // 1. Create a clean temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 2. Create subfolders
    let figures_dir = temp_path.join("People/Genesis 8 Female");
    let clothing_dir = temp_path.join("Clothing/Shirt");
    fs::create_dir_all(&figures_dir).unwrap();
    fs::create_dir_all(&clothing_dir).unwrap();

    // 3. Write mock DUF files
    let figure_file = figures_dir.join("Genesis 8 Female.duf");
    let shirt_file = clothing_dir.join("Sleek Shirt.duf");

    // Write standard JSON contents into mock files
    let figure_json = serde_json::json!({
        "type": "figure",
        "asset_info": {
            "name": "Genesis 8 Female",
            "contributor": "Daz 3D"
        }
    });
    let shirt_json = serde_json::json!({
        "type": "clothing",
        "vendor": "VendorA",
        "asset_info": {
            "name": "Sleek Shirt",
            "contributor": "VendorA"
        }
    });

    fs::write(&figure_file, serde_json::to_string(&figure_json).unwrap()).unwrap();
    fs::write(&shirt_file, serde_json::to_string(&shirt_json).unwrap()).unwrap();

    // 4. Run the scanner
    let result = library_scanner::scan_directory(&temp_path.to_string_lossy());

    // 5. Verify results
    assert_eq!(
        result.total_files, 2,
        "Scanner should discover exactly 2 supported files."
    );

    // Assert figures categorization
    assert!(
        !result.categorized.figures.is_empty(),
        "Genesis 8 Female must be categorized as a figure."
    );
    let fig_asset = &result.categorized.figures[0];
    assert_eq!(fig_asset.name, "Genesis 8 Female");
    assert_eq!(fig_asset.file_type, "duf");

    // Assert clothing categorization
    assert!(
        !result.categorized.clothing.is_empty(),
        "Sleek Shirt must be categorized as clothing."
    );
    let shirt_asset = &result.categorized.clothing[0];
    assert_eq!(shirt_asset.name, "Sleek Shirt");
    assert_eq!(
        shirt_asset.metadata.get("vendor").map(|s| s.as_str()),
        Some("VendorA")
    );
}

#[test]
fn test_get_default_content_paths_runs() {
    // Simply check that the default paths method returns without crashing
    let _paths = library_scanner::get_default_content_paths();
}
