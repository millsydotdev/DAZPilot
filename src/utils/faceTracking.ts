export const FACE_LANDMARKS = {
  L_EYE_TOP: 159,
  L_EYE_BOT: 145,
  L_EYE_OUT: 33,
  L_EYE_IN: 133,
  R_EYE_TOP: 386,
  R_EYE_BOT: 374,
  R_EYE_OUT: 263,
  R_EYE_IN: 362,

  L_BROW_IN: 107,
  L_BROW_MID: 55,
  L_BROW_OUT: 46,
  R_BROW_IN: 336,
  R_BROW_MID: 285,
  R_BROW_OUT: 276,

  MOUTH_L: 61,
  MOUTH_R: 291,
  LIP_UP_IN: 13,
  LIP_DN_IN: 14,

  FACE_TOP: 10,
  FACE_BOT: 152,
};

export const FACS_MAP: Record<string, [string, number]> = {
  eye_blink_l: ['AU 45 Blink Left', 1.0],
  eye_blink_r: ['AU 45 Blink Right', 1.0],
  eye_wide_l: ['Eye Wide Left', 0.8],
  eye_wide_r: ['Eye Wide Right', 0.8],
  brow_inner_up_l: ['AU 01 Inner Brow Raiser Left', 0.9],
  brow_inner_up_r: ['AU 01 Inner Brow Raiser Right', 0.9],
  brow_outer_up_l: ['AU 02 Outer Brow Raiser Left', 0.9],
  brow_outer_up_r: ['AU 02 Outer Brow Raiser Right', 0.9],
  brow_down_l: ['AU 04 Brow Lowerer Left', 0.9],
  brow_down_r: ['AU 04 Brow Lowerer Right', 0.9],
  jaw_open: ['AU 26 Jaw Drop', 1.0],
  mouth_smile_l: ['AU 12 Lip Corner Puller Left', 0.9],
  mouth_smile_r: ['AU 12 Lip Corner Puller Right', 0.9],
  mouth_frown_l: ['AU 15 Lip Corner Depressor Left', 0.9],
  mouth_frown_r: ['AU 15 Lip Corner Depressor Right', 0.9],
};

function dist(a: { x: number; y: number }, b: { x: number; y: number }): number {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

function clamp(v: number, lo: number = 0.0, hi: number = 1.0): number {
  return Math.max(lo, Math.min(hi, v));
}

function ear(
  lm: Array<{ x: number; y: number }>,
  top: number,
  bot: number,
  out: number,
  inn: number
): number {
  const h = dist(lm[out], lm[inn]);
  return h > 0 ? dist(lm[top], lm[bot]) / h : 0.0;
}

export function computeAUs(lm: Array<{ x: number; y: number }>): Record<string, number> {
  const L = FACE_LANDMARKS;
  const face_h = dist(lm[L.FACE_TOP], lm[L.FACE_BOT]) || 1.0;
  const face_w = dist(lm[L.L_EYE_OUT], lm[L.R_EYE_OUT]) || 1.0;
  const aus: Record<string, number> = {};

  const OPEN_EAR = 0.28;
  const l_ear = ear(lm, L.L_EYE_TOP, L.L_EYE_BOT, L.L_EYE_OUT, L.L_EYE_IN);
  const r_ear = ear(lm, L.R_EYE_TOP, L.R_EYE_BOT, L.R_EYE_OUT, L.R_EYE_IN);
  aus.eye_blink_l = clamp(1.0 - l_ear / OPEN_EAR);
  aus.eye_blink_r = clamp(1.0 - r_ear / OPEN_EAR);
  aus.eye_wide_l = clamp((l_ear - OPEN_EAR) / 0.12);
  aus.eye_wide_r = clamp((r_ear - OPEN_EAR) / 0.12);

  const BROW_IN_NEUTRAL = 0.06;
  const BROW_IN_RANGE = 0.05;
  aus.brow_inner_up_l = clamp(
    ((lm[L.L_EYE_IN].y - lm[L.L_BROW_IN].y) / face_h - BROW_IN_NEUTRAL) / BROW_IN_RANGE
  );
  aus.brow_inner_up_r = clamp(
    ((lm[L.R_EYE_IN].y - lm[L.R_BROW_IN].y) / face_h - BROW_IN_NEUTRAL) / BROW_IN_RANGE
  );

  const BROW_OUT_NEUTRAL = 0.065;
  const BROW_OUT_RANGE = 0.05;
  aus.brow_outer_up_l = clamp(
    ((lm[L.L_EYE_OUT].y - lm[L.L_BROW_OUT].y) / face_h - BROW_OUT_NEUTRAL) / BROW_OUT_RANGE
  );
  aus.brow_outer_up_r = clamp(
    ((lm[L.R_EYE_OUT].y - lm[L.R_BROW_OUT].y) / face_h - BROW_OUT_NEUTRAL) / BROW_OUT_RANGE
  );

  const BROW_CONV_NEUTRAL = 0.4;
  const BROW_CONV_RANGE = 0.14;
  const brow_conv = dist(lm[L.L_BROW_IN], lm[L.R_BROW_IN]) / face_w;
  const brow_down = clamp((BROW_CONV_NEUTRAL - brow_conv) / BROW_CONV_RANGE);
  aus.brow_down_l = brow_down;
  aus.brow_down_r = brow_down;

  const JAW_NEUTRAL = 0.018;
  const JAW_RANGE = 0.09;
  aus.jaw_open = clamp((dist(lm[L.LIP_UP_IN], lm[L.LIP_DN_IN]) / face_h - JAW_NEUTRAL) / JAW_RANGE);

  const SMILE_NEUTRAL = 0.44;
  const SMILE_RANGE = 0.12;
  const mouth_w = dist(lm[L.MOUTH_L], lm[L.MOUTH_R]) / face_w;
  const smile = clamp((mouth_w - SMILE_NEUTRAL) / SMILE_RANGE);
  aus.mouth_smile_l = smile;
  aus.mouth_smile_r = smile;

  const lip_ctr_y = (lm[L.LIP_UP_IN].y + lm[L.LIP_DN_IN].y) / 2.0;
  const FROWN_NEUTRAL = 0.008;
  const FROWN_RANGE = 0.03;
  aus.mouth_frown_l = clamp(((lm[L.MOUTH_L].y - lip_ctr_y) / face_h - FROWN_NEUTRAL) / FROWN_RANGE);
  aus.mouth_frown_r = clamp(((lm[L.MOUTH_R].y - lip_ctr_y) / face_h - FROWN_NEUTRAL) / FROWN_RANGE);

  return aus;
}
