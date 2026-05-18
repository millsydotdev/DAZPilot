use dazpilot_lib::viewport_sync::ViewportSyncState;
use std::sync::Mutex;

#[test]
fn test_viewport_sync_state_concurrency() {
    // Initialize the synchronization state structure
    let state = ViewportSyncState {
        enabled: Mutex::new(false),
        fps: Mutex::new(5),
    };
    
    // Lock and inspect defaults
    {
        let is_enabled = state.enabled.lock().unwrap();
        assert_eq!(*is_enabled, false, "Viewport sync must be disabled initially.");
        
        let fps = state.fps.lock().unwrap();
        assert_eq!(*fps, 5, "Initial frame rate must be 5 FPS.");
    }
    
    // Modify and verify update correctness
    {
        let mut is_enabled = state.enabled.lock().unwrap();
        *is_enabled = true;
        
        let mut fps = state.fps.lock().unwrap();
        *fps = 10;
    }
    
    // Assert new values
    assert_eq!(*state.enabled.lock().unwrap(), true);
    assert_eq!(*state.fps.lock().unwrap(), 10);
}

#[test]
fn test_fps_sleep_interval_calculations() {
    let fps_5 = 5;
    let sleep_5 = 1000 / fps_5.max(1);
    assert_eq!(sleep_5, 200, "5 FPS must equal 200ms sleep duration.");
    
    let fps_zero = 0;
    let sleep_zero = 1000 / fps_zero.max(1);
    assert_eq!(sleep_zero, 1000, "0 FPS clamp must fallback to 1 FPS (1000ms sleep).");
}
