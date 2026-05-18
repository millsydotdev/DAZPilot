# AI Agent System Documentation

## Overview

The DazPilot uses a multi-agent system to handle user requests, manage assets, handle animations, and learn from user behavior. Each agent has a specific responsibility and they communicate via a central message bus.

---

## Agent Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Central AI Orchestrator                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    Core Services Layer                     │  │
│  │  - Context Manager    - Memory System                   │  │
│  │  - Task Queue         - Decision Logger                  │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    Agent Execution Layer                  │  │
│  │                                                           │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │  │
│  │  │ Asset   │ │ Animation │ │ Physics  │ │Conflict │    │  │
│  │  │Selection│ │  Agent   │ │  Agent   │ │Resolution│    │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │  │
│  │                                                           │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │  │
│  │  │ Import  │ │  Render  │ │ Learning│ │ Task    │    │  │
│  │  │  Agent  │ │  Agent   │ │  Agent   │ │ Planner │    │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │  │
│  │                                                           │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │  │
│  │  │  Voice  │ │  Image   │ │  Style   │ │ Scene   │    │  │
│  │  │ Command │ │Reference│ │Transfer │ │Composition│   │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │  │
│  │                                                           │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Agents

### 1. Task Planner Agent

**Purpose**: Parse user commands and decompose into actionable steps

**Responsibilities**:
- Parse natural language input
- Detect intent and entities
- Create execution plan
- Manage task dependencies
- Handle error recovery

**Key Functions**:
```typescript
interface TaskPlannerAgent {
  parseCommand(input: string): ParsedCommand;
  decomposeToSteps(parsed: ParsedCommand): TaskStep[];
  determineDependencies(steps: TaskStep[]): DependencyGraph;
  createExecutionPlan(steps: TaskStep[]): ExecutionPlan;
  handleFailure(error: Error): RecoveryPlan;
}
```

---

### 2. Asset Selection Agent

**Purpose**: Find and recommend appropriate assets from user's library

**Responsibilities**:
- Match user requests to assets
- Check figure compatibility
- Apply user preferences
- Resolve natural language to assets

**Key Functions**:
```typescript
interface AssetSelectionAgent {
  findAssets(criteria: AssetCriteria, userId: string): Asset[];
  checkCompatibility(asset: Asset, figure: Figure): CompatibilityResult;
  applyPreferences(assets: Asset[], userId: string): Asset[];
  resolvePhraseToAssets(phrase: string, userId: string): Asset[];
}
```

---

### 3. Animation Agent

**Purpose**: Handle pose application, keyframe creation, and animation generation

**Responsibilities**:
- Apply poses to figures
- Generate keyframes
- Handle timeline management
- Create animation sequences
- Convert poses to keyframes

**Key Functions**:
```typescript
interface AnimationAgent {
  applyPose(figure: Figure, pose: Pose): Result;
  createKeyframe(figure: Figure, frame: number, properties: KeyframeProperties): Keyframe;
  createAnimationSequence(steps: AnimationStep[]): Animation;
  posesToKeyframes(pose: Pose, frameRange: Range): Keyframe[];
}
```

---

### 4. Physics Simulation Agent

**Purpose**: Manage dForce physics and collision systems

**Responsibilities**:
- Configure dForce parameters
- Set up colliders
- Run simulation
- Convert results to keyframes

**Key Functions**:
```typescript
interface PhysicsAgent {
  setupSimulation(config: PhysicsConfig): Simulation;
  addColliders(objects: SceneObject[]): ColliderSetup;
  runSimulation(sim: Simulation): SimulationResult;
  convertToKeyframes(result: SimulationResult): Keyframe[];
}
```

---

### 5. Conflict Resolution Agent

**Purpose**: Detect and resolve asset conflicts

**Responsibilities**:
- Detect shell/body zone conflicts
- Identify material channel conflicts
- Analyze bone weight conflicts
- Propose resolution strategies

**Key Functions**:
```typescript
interface ConflictResolutionAgent {
  detectConflicts(scene: Scene): Conflict[];
  analyzeImpact(conflict: Conflict): ImpactAnalysis;
  proposeSolutions(conflict: Conflict): Solution[];
  applySolution(solution: Solution): Result;
}
```

---

### 6. Import/Rigging Agent

**Purpose**: Handle 3D model import and automatic rigging

**Responsibilities**:
- Detect file format
- Analyze geometry
- Generate skeleton
- Calculate weights
- Convert to Daz3D asset

**Key Functions**:
```typescript
interface ImportAgent {
  detectFormat(file: string): Format;
  analyzeGeometry(mesh: Mesh): GeometryAnalysis;
  generateSkeleton(analysis: GeometryAnalysis): Skeleton;
  calculateWeights(mesh: Mesh, skeleton: Skeleton): WeightMap[];
  convertToAsset(processed: ProcessedModel): DazAsset;
}
```

---

### 7. Render Agent

**Purpose**: Manage rendering and export

**Responsibilities**:
- Configure render settings
- Execute renders
- Handle output formats
- Manage batch renders

**Key Functions**:
```typescript
interface RenderAgent {
  configureRender(settings: RenderSettings): Config;
  executeRender(config: Config, scene: Scene): RenderResult;
  exportToFormat(result: RenderResult, format: ExportFormat): OutputFile;
  queueRender(render: RenderJob): QueuePosition;
}
```

---

### 8. Learning Agent

**Purpose**: Track user behavior and improve suggestions

**Responsibilities**:
- Log AI decisions
- Track user responses
- Calculate accuracy
- Detect patterns

**Key Functions**:
```typescript
interface LearningAgent {
  logDecision(decision: AIDecision): void;
  trackResponse(decisionId: string, response: UserResponse): void;
  calculateAccuracy(): AccuracyMetrics;
  detectPatterns(): WorkflowPattern[];
  updatePreferences(userId: string): UserPreferences;
}
```

---

### 9. Voice Command Agent

**Purpose**: Convert speech to actionable commands

**Responsibilities**:
- Speech-to-text conversion
- Command interpretation
- Execute voice commands

**Key Functions**:
```typescript
interface VoiceAgent {
  captureAudio(): AudioBuffer;
  transcribe(audio: AudioBuffer): string;
  interpret(text: string): VoiceCommand;
  execute(command: VoiceCommand): Result;
}
```

---

### 10. Image Reference Agent

**Purpose**: Analyze reference images for style matching

**Responsibilities**:
- Accept image input
- Analyze style
- Match to assets
- Suggest compositions

**Key Functions**:
```typescript
interface ImageAgent {
  processImage(image: ImageBuffer): ImageAnalysis;
  detectStyle(analysis: ImageAnalysis): StyleProfile;
  findMatchingAssets(style: StyleProfile, userId: string): Asset[];
  suggestComposition(analysis: ImageAnalysis): CompositionSuggestion;
}
```

---

### 11. Style Transfer Agent

**Purpose**: Apply artistic styles to materials and scenes

**Responsibilities**:
- Detect source style
- Apply to target
- Blend styles

**Key Functions**:
```typescript
interface StyleAgent {
  analyzeStyle(source: StyleSource): StyleProfile;
  applyStyle(target: Material, style: StyleProfile): Material;
  blendStyles(styles: StyleProfile[], target: Material): Material;
}
```

---

### 12. Scene Composition Agent

**Purpose**: Auto-arrange scene elements

**Responsibilities**:
- Analyze scene requirements
- Position elements
- Configure lighting
- Frame camera

**Key Functions**:
```typescript
interface SceneCompositionAgent {
  analyzeScene(requirements: SceneRequirements): SceneAnalysis;
  composeElements(analysis: SceneAnalysis): SceneLayout;
  setupLighting(layout: SceneLayout): LightingSetup;
  frameCamera(layout: SceneLayout): CameraSettings;
}
```

---

## Communication Protocol

### Message Types

```typescript
enum AgentMessageType {
  REQUEST = "request",      // Agent requests something
  RESPONSE = "response",   // Agent responds to request
  BROADCAST = "broadcast", // Agent notifies others
  ERROR = "error",         // Error occurred
  COMPLETE = "complete"    // Task completed
}
```

### Agent Communication Flow

```
Agent A (Sender)
   │
   ▼
Message Bus
   │
   ├──► Agent B (Receiver)
   ├──► Agent C (Receiver)
   └──► Agent D (Receiver)
```

---

## Task Execution

### Task Structure

```typescript
interface Task {
  id: string;
  command: string;
  intent: ParsedIntent;
  steps: TaskStep[];
  assignedAgents: string[];
  status: TaskStatus;
  result?: TaskResult;
}
```

### Step Execution

```typescript
interface TaskStep {
  id: number;
  agent: string;          // Which agent handles this
  action: string;         // What to do
  params: object;          // Action parameters
  dependsOn: number[];     // Step dependencies
  status: StepStatus;
}
```

---

## Agent Performance Metrics

Each agent tracks:

| Metric | Description |
|--------|-------------|
| Total Executions | How many times agent ran |
| Success Rate | Percentage of successful runs |
| Average Time | Mean execution time |
| Confidence Score | Average confidence of outputs |
| Last Executed | Timestamp of last run |

---

## Error Handling

### Recovery Strategies

1. **Fallback Agent** - If primary agent fails, try alternative
2. **Retry Logic** - Attempt failed action again
3. **Human Escalation** - Request user input when stuck
4. **Partial Execution** - Complete what's possible, report what's not

---

## Adding New Agents

To add a new agent:

1. Create agent class in `src-tauri/src/agents/`
2. Implement agent interface
3. Register in AgentManager
4. Add to agent configuration
5. Write tests

---

Last Updated: May 2026
Status: Documentation Complete
