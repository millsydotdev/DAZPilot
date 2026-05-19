import { useId } from 'react';
import { cn } from '../../utils/cn';

interface DazPilotLogoProps {
  className?: string;
  size?: number;
  showWordmark?: boolean;
  compact?: boolean;
}

export function DazPilotLogo({
  className = '',
  size = 48,
  showWordmark = false,
  compact = false,
}: DazPilotLogoProps) {
  const uid = useId().replace(/:/g, '');
  const crystalId = `dp-crystal-${uid}`;
  const glowId = `dp-glow-${uid}`;

  return (
    <div className={cn('flex items-center gap-3', className)}>
      <svg
        width={size}
        height={size}
        viewBox="0 0 64 64"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        aria-hidden
      >
        <defs>
          <linearGradient
            id={crystalId}
            x1="8"
            y1="8"
            x2="56"
            y2="56"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="#ef4444" />
            <stop offset="1" stopColor="#dc2626" />
          </linearGradient>
          <filter id={glowId} x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="2" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>
        <path
          d="M32 4L54 18V46L32 60L10 46V18L32 4Z"
          stroke={`url(#${crystalId})`}
          strokeWidth="1.5"
          fill="rgba(239,68,68,0.08)"
          filter={`url(#${glowId})`}
        />
        <path
          d="M32 14L44 22V42L32 50L20 42V22L32 14Z"
          stroke="#f87171"
          strokeWidth="1"
          fill="rgba(239,68,68,0.15)"
        />
        <path
          d="M32 22V42M20 32H44M24 24L40 40M40 24L24 40"
          stroke="#22d3ee"
          strokeWidth="0.75"
          strokeOpacity="0.5"
          strokeLinecap="round"
        />
        <ellipse cx="32" cy="30" rx="6" ry="7" fill="rgba(250,250,250,0.9)" />
        <path
          d="M28 28C28 26 30 25 32 25C34 25 36 26 36 28"
          stroke="#0a0a0a"
          strokeWidth="0.75"
          fill="none"
        />
        <circle cx="30" cy="29" r="0.75" fill="#0a0a0a" />
        <circle cx="34" cy="29" r="0.75" fill="#0a0a0a" />
      </svg>
      {showWordmark && !compact && (
        <div className="flex flex-col leading-none">
          <span className="font-display text-xl font-bold tracking-tight text-white">
            Daz<span className="text-accent">Pilot</span>
          </span>
          <span className="mt-1 font-mono text-[10px] uppercase tracking-[0.2em] text-zinc-500">
            Scene Control
          </span>
        </div>
      )}
    </div>
  );
}

export function DazPilotLogoCompact({
  className = '',
  size = 32,
}: Pick<DazPilotLogoProps, 'className' | 'size'>) {
  return <DazPilotLogo className={className} size={size} compact />;
}
