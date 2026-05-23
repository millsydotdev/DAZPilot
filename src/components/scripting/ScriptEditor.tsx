import React, { useState, useRef, useEffect } from 'react';
import { useToastStore } from '../../store/toastStore';
import { useScriptApprovalStore } from '../../store/scriptApprovalStore';
import { cn } from '../../utils/cn';
import MonacoEditor from '@monaco-editor/react';

interface ScriptEditorProps {
  initialScript?: string;
  onScriptExecuted?: (result: string) => void;
  onScriptSaved?: (script: string) => void;
}

const ScriptEditor: React.FC<ScriptEditorProps> = ({
  initialScript = '',
  onScriptExecuted,
  onScriptSaved,
}) => {
  const [script, setScript] = useState(initialScript);
  const [isLoading, setIsLoading] = useState(false);
  const [executionResult, setExecutionResult] = useState<string | null>(null);
  const [aiSuggestion, setAISuggestion] = useState<string | null>(null);
  const [invoke, setInvoke] = useState<
    ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null
  >(null);
  const editorRef = useRef<{
    getModel: () => { setValue: (v: string) => void } | null;
    focus: () => void;
  } | null>(null);

  const toast = useToastStore.getState();
  const { addSuggestion } = useScriptApprovalStore();

  // Load Tauri invoke function
  useEffect(() => {
    import('@tauri-apps/api/core').then(({ invoke }) => {
      setInvoke(invoke);
    });
  }, []);

  useEffect(() => {
    if (initialScript) {
      setScript((prev) => (prev === '' ? initialScript : prev));
    }
  }, [initialScript]);

  const handleScriptChange = (value: string | undefined) => {
    setScript(value ?? '');
  };

  const handleEditorDidMount = (editor: {
    updateOptions: (opts: Record<string, unknown>) => void;
  }) => {
    editorRef.current = editor as unknown as {
      getModel: () => { setValue: (v: string) => void } | null;
      focus: () => void;
    };
    editor.updateOptions({
      wordWrap: 'on',
      minimap: { enabled: false },
    });
  };

  const executeScript = async () => {
    if (!script.trim()) {
      toast.warning('Please enter a script to execute', 3000);
      return;
    }

    if (!invoke) {
      toast.error('Backend not ready', 3000);
      return;
    }

    setIsLoading(true);
    try {
      const result = await invoke('execute_approved_script', { script });
      const resultStr = String(result);
      setExecutionResult(resultStr);
      if (onScriptExecuted) onScriptExecuted(resultStr);

      // Add to script approval history for tracking
      addSuggestion({
        id: Date.now().toString(),
        script,
        context: 'Manual execution',
        timestamp: new Date().toISOString(),
      });

      toast.success('Script executed successfully', 3000);
    } catch (error) {
      setExecutionResult(`Error: ${error}`);
      toast.error(`Script execution failed: ${error}`, 5000);
    } finally {
      setIsLoading(false);
    }
  };

  const clearEditor = () => {
    setScript('');
    setExecutionResult(null);
    if (editorRef.current) {
      editorRef.current.getModel()?.setValue('');
    }
  };

  const saveScript = async () => {
    if (!script.trim()) {
      toast.warning('Please enter a script to save', 3000);
      return;
    }

    if (onScriptSaved) {
      onScriptSaved(script);
      toast.success('Script saved', 3000);
    }
  };

  const requestAISuggestion = async () => {
    // This would integrate with your AI service to generate script suggestions
    // For now, we'll show a placeholder
    setAISuggestion(
      `// AI-generated script suggestion based on context\n// Describe what you want to achieve and I'll generate the DazScript for you\n\n// Example: Load a Genesis 8 female figure\nApp.getContentMgr().openFile("C:/Users/Public/Documents/My DAZ 3D Library/Genesis 8 Female/Genesis8Female.dsf", true);\n\n// Example: Apply a pose\nApp.getContentMgr().openFile("C:/Users/Public/Documents/My DAZ 3D Library/Poses/MyPose.dsf", true);`
    );

    toast.info('AI suggestion generated! Check the suggestions panel.', 3000);
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-col md:flex-row md:items-start md:space-x-4">
        <div className="flex-1 min-w-0">
          <div className="flex justify-between items-center mb-2">
            <h3 className="text-lg font-semibold">Script Editor</h3>
            <div className="flex space-x-2">
              <button
                onClick={executeScript}
                disabled={isLoading}
                className={cn(
                  'px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50',
                  isLoading && 'cursor-not-allowed'
                )}
              >
                {isLoading ? 'Executing...' : 'Execute Script'}
              </button>
              <button
                onClick={clearEditor}
                className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700"
              >
                Clear
              </button>
              <button
                onClick={saveScript}
                className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
              >
                Save Script
              </button>
              <button
                onClick={requestAISuggestion}
                className="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700"
              >
                AI Suggestion
              </button>
            </div>
          </div>

          <div className="relative">
            <MonacoEditor
              height="400px"
              defaultLanguage="javascript"
              theme="vs-dark"
              value={script}
              onChange={handleScriptChange}
              onMount={handleEditorDidMount}
              width="100%"
              options={{
                scrollBeyondLastLine: false,
                readOnly: false,
                automaticLayout: true,
              }}
            />

            {/* Line numbers and gutter */}
            <div className="absolute inset-0 pointer-events-none">
              <div className="flex h-full items-start">
                <div className="w-12 flex-shrink-0 bg-gray-800 text-gray-400 text-xs px-2 pt-1">
                  {[...Array(20)].map((_, i) => (
                    <div key={i} className="h-5 flex items-center justify-end">
                      {i + 1}
                    </div>
                  ))}
                </div>
                <div className="flex-1"></div>
              </div>
            </div>
          </div>
        </div>

        <div className="w-64 md:w-80 space-y-4">
          {executionResult && (
            <div className="bg-gray-800 rounded p-4">
              <h4 className="font-semibold mb-2">Execution Result:</h4>
              <pre className="text-sm text-green-400 bg-gray-900 p-2 rounded overflow-auto">
                {executionResult}
              </pre>
            </div>
          )}

          {aiSuggestion && (
            <div className="bg-purple-900/20 border border-purple-500 rounded p-4">
              <h4 className="font-semibold mb-2 flex items-center">
                <svg
                  className="w-5 h-5 text-purple-400 mr-2"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth="2"
                    d="M13 10V3L4 14h7v7l9-11h-7z"
                  />
                </svg>
                AI Suggestion
              </h4>
              <pre className="text-sm text-purple-300 bg-purple-900 p-2 rounded overflow-auto max-h-[200px]">
                {aiSuggestion}
              </pre>
              <div className="mt-3 flex justify-end">
                <button
                  onClick={() => {
                    setScript(aiSuggestion || '');
                    setAISuggestion(null);
                  }}
                  className="px-3 py-1 bg-purple-600 text-white text-xs rounded hover:bg-purple-700"
                >
                  Use Suggestion
                </button>
                <button
                  onClick={() => setAISuggestion(null)}
                  className="ml-2 px-3 py-1 bg-gray-600 text-white text-xs rounded hover:bg-gray-700"
                >
                  Dismiss
                </button>
              </div>
            </div>
          )}

          <div className="bg-gray-800 rounded p-4">
            <h4 className="font-semibold mb-2">Quick Insert</h4>
            <div className="space-y-2">
              <button
                onClick={() => {
                  setScript(
                    (s) =>
                      s + '// Load Asset\nApp.getContentMgr().openFile("PATH_TO_FILE", true);\n\n'
                  );
                  if (editorRef.current) {
                    editorRef.current.focus();
                  }
                }}
                className="w-full text-left bg-gray-700 text-white py-2 px-3 rounded hover:bg-gray-600"
              >
                Load Asset
              </button>
              <button
                onClick={() => {
                  setScript(
                    (s) =>
                      s + '// Apply Pose\nApp.getContentMgr().openFile("PATH_TO_POSE", true);\n\n'
                  );
                  if (editorRef.current) {
                    editorRef.current.focus();
                  }
                }}
                className="w-full text-left bg-gray-700 text-white py-2 px-3 rounded hover:bg-gray-600"
              >
                Apply Pose
              </button>
              <button
                onClick={() => {
                  setScript(
                    (s) =>
                      s +
                      '// Select Node\nScene.setPrimarySelection(Scene.findNode("NODE_NAME"));\n\n'
                  );
                  if (editorRef.current) {
                    editorRef.current.focus();
                  }
                }}
                className="w-full text-left bg-gray-700 text-white py-2 px-3 rounded hover:bg-gray-600"
              >
                Select Node
              </button>
              <button
                onClick={() => {
                  setScript(
                    (s) =>
                      s +
                      '// Capture Viewport\nvar img = App.getActiveViewport().get3DViewport().captureImage();\nimg.save("PATH_TO_SAVE");\n\n'
                  );
                  if (editorRef.current) {
                    editorRef.current.focus();
                  }
                }}
                className="w-full text-left bg-gray-700 text-white py-2 px-3 rounded hover:bg-gray-600"
              >
                Capture Viewport
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Status bar */}
      <div className="border-t border-gray-700 pt-2 text-xs text-gray-400 flex justify-between">
        <span>Language: JavaScript (DazScript)</span>
        <span>Lines: {script.split('\n').length}</span>
        <span>Characters: {script.length}</span>
      </div>
    </div>
  );
};

export default ScriptEditor;
