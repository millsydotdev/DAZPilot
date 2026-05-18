import { useEffect, useRef, useState } from 'react';
import { FaceLandmarker, FilesetResolver } from '@mediapipe/tasks-vision';
import { invoke } from '@tauri-apps/api/core';
import { computeAUs, FACS_MAP } from '../../utils/faceTracking';
import { X, Camera, RefreshCw } from 'lucide-react';
import { Button } from '../ui';
import styles from './LiveLinkPanel.module.css';

const pushFacsToDaz = async (aus: Record<string, number>, scale: number = 1.0) => {
  const targets: Record<string, number> = {};
  for (const [key, [label, auScale]] of Object.entries(FACS_MAP)) {
    targets[label] = Number(Math.max(0, Math.min(1.0, (aus[key] || 0) * auScale * scale)).toFixed(4));
  }

  const script = `(function(){
    var _skel=null,_skels=Scene.getSkeletonList();
    for(var i=0;i<_skels.length;i++){
      if(_skels[i].getLabel()==="Genesis 9"){_skel=_skels[i];break;}
    }
    if(!_skel) return false;
    var t = ${JSON.stringify(targets)};
    for(var i=0; i<_skel.getNumProperties(); i++){
      var p = _skel.getProperty(i);
      if(p && p.getLabel && t.hasOwnProperty(p.getLabel())){
        p.setValue(t[p.getLabel()]);
      }
    }
    return true;
  })()`;

  await invoke('send_daz3d_command', {
    command: 'run_script',
    args: { script, args: {} }
  });
};

const zeroFacs = async () => {
  try {
    await pushFacsToDaz({}, 0.0);
  } catch (e) {
    // Ignore unmount push failures
  }
};

interface LiveLinkPanelProps {
  onClose: () => void;
}

export default function LiveLinkPanel({ onClose }: LiveLinkPanelProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isActive, setIsActive] = useState(false);
  const [isModelLoading, setIsModelLoading] = useState(true);
  const faceLandmarkerRef = useRef<FaceLandmarker | null>(null);
  const requestRef = useRef<number>();
  const lastUpdateRef = useRef<number>(0);
  const emaAusRef = useRef<Record<string, number>>({});
  const lastDazPushRef = useRef<number>(0);

  useEffect(() => {
    async function loadModel() {
      try {
        const vision = await FilesetResolver.forVisionTasks(
          "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@latest/wasm"
        );
        faceLandmarkerRef.current = await FaceLandmarker.createFromOptions(vision, {
          baseOptions: {
            modelAssetPath: "https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task",
            delegate: "GPU"
          },
          outputFaceBlendshapes: false,
          runningMode: "VIDEO",
          numFaces: 1,
        });
        setIsModelLoading(false);
      } catch (err) {
        console.error("Failed to load MediaPipe model:", err);
      }
    }
    loadModel();

    return () => {
      if (faceLandmarkerRef.current) {
        faceLandmarkerRef.current.close();
      }
      if (requestRef.current) cancelAnimationFrame(requestRef.current);
      stopCamera();
      zeroFacs();
    };
  }, []);

  function stopCamera() {
    if (videoRef.current && videoRef.current.srcObject) {
      const stream = videoRef.current.srcObject as MediaStream;
      stream.getTracks().forEach(t => t.stop());
      videoRef.current.srcObject = null;
    }
  }

  const startLiveLink = async () => {
    if (!faceLandmarkerRef.current) return;
    
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ 
        video: { width: 640, height: 480, facingMode: 'user' } 
      });
      if (videoRef.current) {
        videoRef.current.srcObject = stream;
        videoRef.current.play();
        setIsActive(true);
      }
    } catch (err) {
      console.error("Failed to access camera", err);
    }
  };

  const stopLiveLink = () => {
    setIsActive(false);
    stopCamera();
    zeroFacs();
  };

  useEffect(() => {
    let active = true;

    const loop = async () => {
      if (!active || !videoRef.current || !faceLandmarkerRef.current || !isActive) return;

      const video = videoRef.current;
      if (video.currentTime !== lastUpdateRef.current) {
        lastUpdateRef.current = video.currentTime;
        
        const startTimeMs = performance.now();
        const results = faceLandmarkerRef.current.detectForVideo(video, startTimeMs);

        if (canvasRef.current) {
          const ctx = canvasRef.current.getContext('2d');
          if (ctx) {
            ctx.save();
            ctx.clearRect(0, 0, canvasRef.current.width, canvasRef.current.height);
            
            if (results.faceLandmarks && results.faceLandmarks.length > 0) {
              const lms = results.faceLandmarks[0].map(lm => ({ x: lm.x, y: lm.y }));
              
              // Draw simple debug dots
              ctx.fillStyle = '#00ff00';
              for (const lm of lms) {
                ctx.beginPath();
                ctx.arc(lm.x * canvasRef.current.width, lm.y * canvasRef.current.height, 1.5, 0, 2 * Math.PI);
                ctx.fill();
              }

              // Compute AUs
              const rawAus = computeAUs(lms);
              
              // EMA Smoothing
              const alpha = 0.5; // smoothing factor
              const emaAus = emaAusRef.current;
              for (const k in rawAus) {
                emaAus[k] = alpha * (emaAus[k] || rawAus[k]) + (1 - alpha) * rawAus[k];
              }
              emaAusRef.current = emaAus;

              // Send to Daz Studio (limit to ~10-15 fps to avoid choking the bridge)
              if (startTimeMs - lastDazPushRef.current > 80 || !lastDazPushRef.current) {
                lastDazPushRef.current = startTimeMs;
                pushFacsToDaz(emaAus);
              }
            }
            ctx.restore();
          }
        }
      }

      if (isActive && active) {
        requestRef.current = requestAnimationFrame(loop);
      }
    };

    if (isActive) {
      requestRef.current = requestAnimationFrame(loop);
    }
    return () => {
      active = false;
      if (requestRef.current) cancelAnimationFrame(requestRef.current);
    };
  }, [isActive]);

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <div className={styles.title}>
          <Camera size={16} className={styles.icon} />
          Live Link
        </div>
        <button onClick={onClose} className={styles.closeBtn}>
          <X size={16} />
        </button>
      </div>

      <div className={styles.content}>
        <div className={styles.previewContainer}>
          <video 
            ref={videoRef} 
            className={styles.video} 
            autoPlay 
            playsInline 
            muted
            style={{ transform: 'scaleX(-1)' }} // Mirror view
          />
          <canvas 
            ref={canvasRef} 
            className={styles.canvas} 
            width={640} 
            height={480}
            style={{ transform: 'scaleX(-1)' }} // Mirror canvas
          />
          {isActive && (
            <>
              <div className={styles.hudScanline} />
              <div className={`${styles.hudBracket} ${styles.topLeft}`} />
              <div className={`${styles.hudBracket} ${styles.topRight}`} />
              <div className={`${styles.hudBracket} ${styles.bottomLeft}`} />
              <div className={`${styles.hudBracket} ${styles.bottomRight}`} />
              <div className={styles.hudStatus}>
                <span className={styles.hudStatusDot} />
                <span>LIVE LINK ACTIVE</span>
              </div>
            </>
          )}
          {isModelLoading && (
            <div className={styles.loadingOverlay}>
              <RefreshCw className={styles.spinner} />
              <span className={styles.loadingText}>Initializing Face Landmarkers...</span>
            </div>
          )}
        </div>

        <div className={styles.controls}>
          <Button 
            onClick={isActive ? stopLiveLink : startLiveLink}
            disabled={isModelLoading}
            variant={isActive ? 'secondary' : 'primary'}
            className={styles.mainBtn}
          >
            {isActive ? 'Stop Live Link' : 'Start Live Link'}
          </Button>
          <p className={styles.hint}>
            {"Mirrors your webcam to \"Genesis 9\" in the active scene."}
          </p>
        </div>
      </div>
    </div>
  );
}
