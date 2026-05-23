import { describe, it, expect } from 'vitest';
import { computeAUs, FACE_LANDMARKS, FACS_MAP } from './faceTracking';

function defaultLandmarks(): Array<{ x: number; y: number }> {
  const lms: Array<{ x: number; y: number }> = [];
  const maxIndex = Math.max(...Object.values(FACE_LANDMARKS));
  for (let i = 0; i <= maxIndex; i++) {
    lms.push({ x: 0, y: 0 });
  }
  const L = FACE_LANDMARKS;

  lms[L.FACE_TOP] = { x: 0, y: 0 };
  lms[L.FACE_BOT] = { x: 0, y: 200 };

  lms[L.L_EYE_OUT] = { x: 80, y: 50 };
  lms[L.R_EYE_OUT] = { x: 180, y: 50 };

  lms[L.L_EYE_TOP] = { x: 90, y: 45 };
  lms[L.L_EYE_BOT] = { x: 90, y: 55 };
  lms[L.L_EYE_IN] = { x: 100, y: 50 };
  lms[L.R_EYE_TOP] = { x: 160, y: 45 };
  lms[L.R_EYE_BOT] = { x: 160, y: 55 };
  lms[L.R_EYE_IN] = { x: 170, y: 50 };

  lms[L.L_BROW_IN] = { x: 90, y: 38 };
  lms[L.L_BROW_OUT] = { x: 80, y: 38 };
  lms[L.R_BROW_IN] = { x: 160, y: 38 };
  lms[L.R_BROW_OUT] = { x: 170, y: 38 };

  lms[L.MOUTH_L] = { x: 90, y: 120 };
  lms[L.MOUTH_R] = { x: 170, y: 120 };
  lms[L.LIP_UP_IN] = { x: 130, y: 115 };
  lms[L.LIP_DN_IN] = { x: 130, y: 125 };

  return lms;
}

describe('FACE_LANDMARKS', () => {
  it('contains expected landmark indices', () => {
    expect(FACE_LANDMARKS.L_EYE_TOP).toBe(159);
    expect(FACE_LANDMARKS.FACE_BOT).toBe(152);
  });
});

describe('FACS_MAP', () => {
  it('contains known AU entries', () => {
    expect(FACS_MAP.eye_blink_l).toEqual(['AU 45 Blink Left', 1.0]);
    expect(FACS_MAP.jaw_open).toEqual(['AU 26 Jaw Drop', 1.0]);
  });
});

describe('computeAUs', () => {
  it('returns all expected AU keys', () => {
    const aus = computeAUs(defaultLandmarks());
    const expectedKeys = [
      'eye_blink_l',
      'eye_blink_r',
      'eye_wide_l',
      'eye_wide_r',
      'brow_inner_up_l',
      'brow_inner_up_r',
      'brow_outer_up_l',
      'brow_outer_up_r',
      'brow_down_l',
      'brow_down_r',
      'jaw_open',
      'mouth_smile_l',
      'mouth_smile_r',
      'mouth_frown_l',
      'mouth_frown_r',
    ];
    for (const key of expectedKeys) {
      expect(aus).toHaveProperty(key);
    }
  });

  it('clamps AU values between 0 and 1', () => {
    const aus = computeAUs(defaultLandmarks());
    for (const val of Object.values(aus)) {
      expect(val).toBeGreaterThanOrEqual(0);
      expect(val).toBeLessThanOrEqual(1);
    }
  });

  it('detects wide eyes when EAR is high', () => {
    const lms = defaultLandmarks();
    const L = FACE_LANDMARKS;
    lms[L.L_EYE_TOP] = { x: 90, y: 30 };
    lms[L.L_EYE_BOT] = { x: 90, y: 70 };
    lms[L.R_EYE_TOP] = { x: 160, y: 30 };
    lms[L.R_EYE_BOT] = { x: 160, y: 70 };
    const aus = computeAUs(lms);
    expect(aus.eye_wide_l).toBeGreaterThan(0);
    expect(aus.eye_wide_r).toBeGreaterThan(0);
  });

  it('detects blink when eyes are closed', () => {
    const lms = defaultLandmarks();
    const L = FACE_LANDMARKS;
    lms[L.L_EYE_TOP] = { x: 90, y: 50 };
    lms[L.L_EYE_BOT] = { x: 90, y: 50 };
    lms[L.R_EYE_TOP] = { x: 160, y: 50 };
    lms[L.R_EYE_BOT] = { x: 160, y: 50 };
    const aus = computeAUs(lms);
    expect(aus.eye_blink_l).toBeGreaterThan(0.9);
    expect(aus.eye_blink_r).toBeGreaterThan(0.9);
  });

  it('detects jaw open', () => {
    const lms = defaultLandmarks();
    const L = FACE_LANDMARKS;
    lms[L.LIP_UP_IN] = { x: 130, y: 100 };
    lms[L.LIP_DN_IN] = { x: 130, y: 150 };
    const aus = computeAUs(lms);
    expect(aus.jaw_open).toBeGreaterThan(0);
  });

  it('detects smile when mouth corners are wide', () => {
    const lms = defaultLandmarks();
    const L = FACE_LANDMARKS;
    lms[L.MOUTH_L] = { x: 50, y: 120 };
    lms[L.MOUTH_R] = { x: 210, y: 120 };
    const aus = computeAUs(lms);
    expect(aus.mouth_smile_l).toBeGreaterThan(0);
    expect(aus.mouth_smile_r).toBeGreaterThan(0);
  });

  it('detects frown when mouth corners are low', () => {
    const lms = defaultLandmarks();
    const L = FACE_LANDMARKS;
    lms[L.MOUTH_L] = { x: 90, y: 130 };
    lms[L.MOUTH_R] = { x: 170, y: 130 };
    const aus = computeAUs(lms);
    expect(aus.mouth_frown_l).toBeGreaterThan(0);
    expect(aus.mouth_frown_r).toBeGreaterThan(0);
  });

  it('handles empty landmarks gracefully', () => {
    const aus = computeAUs([]);
    for (const val of Object.values(aus)) {
      expect(val).toBeGreaterThanOrEqual(0);
      expect(val).toBeLessThanOrEqual(1);
    }
  });

  it('handles all-zero landmarks gracefully', () => {
    const L = FACE_LANDMARKS;
    const maxIndex = Math.max(...Object.values(L));
    const lms = Array.from({ length: maxIndex + 1 }, () => ({ x: 0, y: 0 }));
    const aus = computeAUs(lms);
    for (const val of Object.values(aus)) {
      expect(val).toBeGreaterThanOrEqual(0);
      expect(val).toBeLessThanOrEqual(1);
    }
  });

  it('handles extreme landmark values', () => {
    const L = FACE_LANDMARKS;
    const maxIndex = Math.max(...Object.values(L));
    const lms = Array.from({ length: maxIndex + 1 }, () => ({ x: 1e6, y: -1e6 }));
    const aus = computeAUs(lms);
    for (const val of Object.values(aus)) {
      expect(val).toBeGreaterThanOrEqual(0);
      expect(val).toBeLessThanOrEqual(1);
    }
  });
});
