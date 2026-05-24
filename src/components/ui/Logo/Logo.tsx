import { DAZPilotLogo, DAZPilotLogoCompact } from '../../brand/DAZPilotLogo';

interface LogoProps {
  className?: string;
  size?: number;
}

export function Logo({ className = '', size = 32 }: LogoProps) {
  return <DAZPilotLogo className={className} size={size} />;
}

export function LogoCompact({ className = '', size = 24 }: LogoProps) {
  return <DAZPilotLogoCompact className={className} size={size} />;
}
