interface KnolLogoProps {
  className?: string;
  label?: string;
}

export default function KnolLogo({ className = 'w-8 h-8', label = 'Knol logo' }: KnolLogoProps) {
  return (
    <svg className={className} viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg" role="img" aria-label={label}>
      <rect width="64" height="64" rx="14" fill="#0A0A0B" />
      <path d="M20 16v32M20 32l16-16M20 32l16 16" stroke="#FAFAFA" strokeWidth="4.5" strokeLinecap="round" strokeLinejoin="round" />
      <circle cx="44" cy="16" r="3.5" fill="#6E56CF" />
      <circle cx="44" cy="48" r="3.5" fill="#6E56CF" />
    </svg>
  );
}
