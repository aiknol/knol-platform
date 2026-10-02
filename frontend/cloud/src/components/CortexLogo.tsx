interface CortexLogoProps {
  className?: string;
  label?: string;
}

export default function CortexLogo({ className = 'w-8 h-8', label = 'DoAide Cortex' }: CortexLogoProps) {
  return (
    <svg className={className} viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg" role="img" aria-label={label}>
      <rect width="64" height="64" rx="14" fill="#0A0A0B" />
      <path d="M12 42L20 22L32 30L44 22L52 42H12Z" fill="#F0B429" opacity="0.9" />
      <circle cx="12" cy="22" r="3.5" fill="#F7CC5F" />
      <circle cx="32" cy="16" r="3.5" fill="#F7CC5F" />
      <circle cx="52" cy="22" r="3.5" fill="#F7CC5F" />
      <rect x="10" y="42" width="44" height="5" rx="2" fill="#F0B429" />
    </svg>
  );
}
