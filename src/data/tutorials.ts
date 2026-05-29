export type TutorialCategory = 'basics' | 'lighting' | 'posing' | 'materials' | 'animation';
export type TutorialDifficulty = 'beginner' | 'intermediate' | 'advanced';

export interface TutorialStepAction {
  command: string;
  args: Record<string, unknown>;
}

export interface TutorialStep {
  id: string;
  title: string;
  concept: string;
  teach: string;
  aiAction?: TutorialStepAction;
  manualInstructions?: string;
  tryYourself?: string;
}

export interface Tutorial {
  id: string;
  title: string;
  description: string;
  category: TutorialCategory;
  difficulty: TutorialDifficulty;
  steps: TutorialStep[];
}

export const tutorials: Tutorial[] = [
  {
    id: 'scene-setup',
    title: 'Scene Setup Basics',
    description:
      'Learn how to add figures, navigate the scene hierarchy, and understand the basic structure of a DAZ Studio scene.',
    category: 'basics',
    difficulty: 'beginner',
    steps: [
      {
        id: 'intro',
        title: 'Welcome',
        concept: 'Scene Composition',
        teach:
          'A DAZ Studio scene is a collection of nodes arranged in a hierarchy. Every figure, light, camera, and prop is a node. Parent nodes affect their children — for example, rotating the hip rotates the entire figure.',
        aiAction: { command: 'add_figure', args: { figure_type: 'genesis9' } },
        manualInstructions:
          'Open DAZ Studio, go to Content Library > People > Genesis 9, double-click "Genesis 9 Female" to load her.',
        tryYourself:
          'In the Scene tab, expand the figure node to see all body parts in the hierarchy.',
      },
      {
        id: 'positioning',
        title: 'Positioning',
        concept: 'Translation & Rotation',
        teach:
          'Every node has X, Y, Z position and rotation properties. You can move figures around the scene by adjusting these. In DAZ Studio, Y is UP, X is left-right, Z is forward-backward.',
        aiAction: {
          command: 'set_property',
          args: { node_id: 'Genesis9', property: 'xRotate', value: '15' },
        },
        manualInstructions:
          'Select the figure, then use the Parameters tab to find xRotate. Drag the slider to rotate.',
        tryYourself: 'Try rotating the figure by changing yRotate to face a different direction.',
      },
      {
        id: 'scene-tree',
        title: 'Scene Tree',
        concept: 'Node Hierarchy',
        teach:
          'The Scene tab in DAZ Studio shows all nodes in a tree. You can parent objects by dragging one onto another. This is crucial for making props follow characters.',
        aiAction: { command: 'add_node', args: { type: 'prop', name: 'Ground_Plane' } },
        manualInstructions:
          'Go to Create > New Primitive > Plane to add a ground plane. Drag it in the Scene tab to position it under the figure.',
        tryYourself: 'Select the figure in the Scene tab and rename it to something custom.',
      },
    ],
  },
  {
    id: 'three-point-lighting',
    title: '3-Point Lighting',
    description:
      'Master the classic 3-point lighting setup — key light, fill light, and rim light — to make your scenes look professional.',
    category: 'lighting',
    difficulty: 'beginner',
    steps: [
      {
        id: 'key-light',
        title: 'Key Light',
        concept: 'The Main Light Source',
        teach:
          'The key light is your primary light. It defines the overall look and determines where shadows fall. Place it at about 45° to the left of the camera, slightly above the subject. This creates natural-looking shadows that give depth.',
        aiAction: {
          command: 'set_light',
          args: { node_id: 'Key_Light', property: 'intensity', value: '80' },
        },
        manualInstructions:
          'Create > New Light > Distant Light. Position it at 45° left and 30° above using the Parameters tab.',
        tryYourself:
          'Move the key light around in the viewport and watch how the shadows change on your figure.',
      },
      {
        id: 'fill-light',
        title: 'Fill Light',
        concept: 'Softening Shadows',
        teach:
          "The fill light goes opposite the key light (right side). It should be less intense — about half the key light's strength — and softer. Its job is to fill in the harsh shadows created by the key light without creating new ones.",
        aiAction: {
          command: 'set_light',
          args: { node_id: 'Fill_Light', property: 'intensity', value: '40' },
        },
        manualInstructions:
          'Add another Distant Light on the right side at 45°. Set its intensity to about 50% of the key light.',
        tryYourself: 'Toggle the fill light on/off to see how it affects shadow depth.',
      },
      {
        id: 'rim-light',
        title: 'Rim / Back Light',
        concept: 'Edge Definition',
        teach:
          'The rim light comes from behind the subject, slightly above. It creates a bright edge (rim) that separates the figure from the background, adding a professional 3D look. This is often the strongest light in a 3-point setup.',
        aiAction: {
          command: 'set_light',
          args: { node_id: 'Rim_Light', property: 'intensity', value: '100' },
        },
        manualInstructions:
          'Add a third Distant Light behind and above the figure. Crank the intensity to 100%.',
        tryYourself: 'Change the rim light color to a warm orange for a sunset rim effect.',
      },
    ],
  },
  {
    id: 'posing-basics',
    title: 'Posing Characters',
    description:
      'Learn how to apply poses, adjust individual joints, and create natural-looking character poses.',
    category: 'posing',
    difficulty: 'beginner',
    steps: [
      {
        id: 'apply-pose',
        title: 'Apply a Pose',
        concept: 'Pose Files (.duf)',
        teach:
          'Poses in DAZ Studio are stored as .duf files containing rotation values for every joint. You can apply pre-made poses from your content library or create your own. DazPilot can search and apply poses for you.',
        aiAction: { command: 'apply_pose', args: { pose: 'Casual Standing' } },
        manualInstructions:
          'In Content Library, browse to People > Genesis 9 > Poses. Double-click a pose preset.',
        tryYourself:
          "After applying the pose, try rotating the figure's head slightly using the Parameters tab.",
      },
      {
        id: 'adjust-joints',
        title: 'Adjust Individual Joints',
        concept: 'Joint Rotation',
        teach:
          'Each joint has X, Y, Z rotation properties. The Parameter dials use degrees — small adjustments (5-15°) make a big difference. The hip controls the whole body, arms control upper body expression.',
        aiAction: {
          command: 'set_property',
          args: { node_id: 'LeftUpperArm', property: 'zRotate', value: '-20' },
        },
        manualInstructions:
          "Select the figure's left arm bone, then in Parameters find zRotate and dial it to -20.",
        tryYourself:
          'Try rotating the right forearm to bring the hand closer to the hip for a more relaxed look.',
      },
      {
        id: 'mirror-pose',
        title: 'Mirroring Poses',
        concept: 'Symmetry',
        teach:
          'When posing, you often want symmetrical adjustments (both arms, both legs). DAZ Studio has a Mirror tool that copies rotation values from one side to the other, optionally inverting them for perfect symmetry.',
        aiAction: undefined,
        manualInstructions:
          'Select a joint on the left side, right-click and choose "Mirror" to copy the pose to the right side.',
        tryYourself:
          'Pose the left arm, then mirror it to the right arm. Adjust individual fingers for more realism.',
      },
    ],
  },
  {
    id: 'materials-101',
    title: 'Material Fundamentals',
    description:
      'Understand DAZ Studio materials — change colors, adjust roughness, and apply textures to make your characters look amazing.',
    category: 'materials',
    difficulty: 'beginner',
    steps: [
      {
        id: 'base-color',
        title: 'Changing Base Color',
        concept: 'Diffuse / Base Color',
        teach:
          "The Base Color (or Diffuse) is the main color of a surface. In DAZ Studio's UberSurface shader, this is the color that appears when the surface is fully lit. You can pick any color or use a texture map.",
        aiAction: {
          command: 'set_material',
          args: { material: 'Skin', property: 'base_color', value: '#f5d0b8' },
        },
        manualInstructions:
          'Select the figure, open the Surfaces tab, find the Skin surface, and click the Base Color swatch to pick a new color.',
        tryYourself:
          'Experiment with different skin tones — try a fantasy color like blue or green.',
      },
      {
        id: 'roughness',
        title: 'Roughness & Shininess',
        concept: 'Surface Finish',
        teach:
          "Roughness controls how shiny a surface is. 0 = mirror-like (wet look), 1 = completely matte (rough). Skin is typically around 0.4-0.6, while metals are 0.1-0.3. Adjusting roughness dramatically changes the material's feel.",
        aiAction: {
          command: 'set_material',
          args: { material: 'Skin', property: 'roughness', value: '0.5' },
        },
        manualInstructions:
          'In the Surfaces tab, find the Roughness slider and drag it. Lower values = shinier.',
        tryYourself:
          'Set roughness to 0.1 for a wet/oily look, then 0.9 for very dry skin. Observe the difference.',
      },
      {
        id: 'textures',
        title: 'Using Textures',
        concept: 'Texture Maps',
        teach:
          'Texture maps are image files (PNG, JPG) that wrap around your 3D model. DAZ Studio uses several types: Diffuse (color), Normal (bump detail), Specular (shininess map), and more. You load them from the Surfaces tab.',
        aiAction: undefined,
        manualInstructions:
          'In Surfaces tab, click the texture folder icon next to Base Color to browse for an image file.',
        tryYourself: 'Find a texture image on your computer and apply it to a clothing surface.',
      },
    ],
  },
  {
    id: 'camera-basics',
    title: 'Camera & Framing',
    description:
      'Learn camera controls, focal length, and composition techniques to frame your renders like a pro.',
    category: 'animation',
    difficulty: 'beginner',
    steps: [
      {
        id: 'camera-types',
        title: 'Camera Types',
        concept: 'DAZ Studio Cameras',
        teach:
          'DAZ Studio has two default cameras: the Viewport camera (for navigating) and render cameras. You can create new cameras to save different angles. Each camera has focal length (zoom), aperture (depth of field), and position.',
        aiAction: { command: 'create_camera', args: { name: 'Hero_Shot' } },
        manualInstructions:
          'Go to Edit > Create New Camera. Name it and position it using the Viewport controls.',
        tryYourself: 'Switch between the new camera and the default camera using the Scene tab.',
      },
      {
        id: 'focal-length',
        title: 'Focal Length',
        concept: 'Zoom & Perspective',
        teach:
          'Focal length is measured in millimeters (mm). Lower values (24-35mm) are wide-angle — they show more of the scene and exaggerate perspective. Higher values (85-200mm) are telephoto — they compress space and are ideal for portraits.',
        aiAction: {
          command: 'set_property',
          args: { node_id: 'Hero_Shot', property: 'focal_length', value: '85' },
        },
        manualInstructions:
          'Select the camera, then in Parameters find "Focal Length" and set it to 85mm for a flattering portrait look.',
        tryYourself:
          'Try 24mm for a dramatic wide shot, then 200mm for a compressed close-up. Notice how the face shape changes.',
      },
      {
        id: 'depth-of-field',
        title: 'Depth of Field',
        concept: 'Aperture & Blur',
        teach:
          'Aperture (f-stop) controls depth of field — how much of the scene is in focus. Lower f-numbers (f/1.4) create shallow depth of field (blurry background). Higher numbers (f/16) keep everything sharp. This is a powerful storytelling tool.',
        aiAction: undefined,
        manualInstructions:
          'Enable Depth of Field in Render Settings. Set the camera aperture to f/2.8 for a soft background blur.',
        tryYourself:
          'Render a test with f/1.4 (shallow DOF) and f/16 (everything sharp). Compare the difference.',
      },
    ],
  },
];
