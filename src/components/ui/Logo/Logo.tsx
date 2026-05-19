import { DazPilotLogo, DazPilotLogoCompact } from '../../brand/DazPilotLogo';

interface LogoProps {
  className?: string;
  size?: number;
}

export function Logo({ className = '', size = 32 }: LogoProps) {
  return <DazPilotLogo className={className} size={size} />;
}

export function LogoCompact({ className = '', size = 24 }: LogoProps) {
  return <DazPilotLogoCompact className={className} size={size} />;
}
