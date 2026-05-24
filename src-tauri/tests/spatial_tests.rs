use dazpilot_lib::vision_service::{generate_spatial_relationships, BoundingBox};

#[test]
fn test_world_space_bounding_box_directions() {
    // A: Genesis 8 Female center [0, 90, 0]
    let a = BoundingBox {
        node_id: "Genesis 8 Female".to_string(),
        min: [-10.0, 0.0, -10.0],
        max: [10.0, 180.0, 10.0],
        center: [0.0, 90.0, 0.0],
    };

    // B: Desk center [60, 45, -30]
    // In Daz Studio: +X is Left (screen-left), +Z is Front (closer to viewer).
    // dx = A - B = 0 - 60 = -60 (< -15 -> A is to the right of B)
    // dy = A - B = 90 - 45 = 45 (> 15 -> A is above B)
    // dz = A - B = 0 - (-30) = 30 (> 15 -> A is in front of B)
    let b = BoundingBox {
        node_id: "Desk".to_string(),
        min: [50.0, 0.0, -40.0],
        max: [70.0, 90.0, -20.0],
        center: [60.0, 45.0, -30.0],
    };

    let bounds = vec![a, b];
    let relationships = generate_spatial_relationships(&bounds);

    println!("Calculated relationships:\n{}", relationships);

    // Assert spatial relationships
    assert!(
        relationships
            .contains("'Genesis 8 Female' is to the right of and in front of and above 'Desk'"),
        "Relative directions should map perfectly to Daz Coordinate system conventions."
    );
    assert!(
        relationships.contains("'Desk' is to the left of and behind and below 'Genesis 8 Female'"),
        "Reciprocal relative directions must match reciprocal vectors exactly."
    );
}
