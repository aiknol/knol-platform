interface PricingCardProps {
  name: string;
  price: string;
  period?: string;
  description: string;
  features: string[];
  highlighted?: boolean;
  cta: string;
  ctaLink: string;
}

export default function PricingCard({
  name, price, period, description, features, highlighted, cta, ctaLink,
}: PricingCardProps) {
  return (
    <div className={`card flex flex-col ${highlighted ? 'border-brand-500/50 ring-1 ring-brand-500/20' : ''}`}>
      {highlighted && (
        <div className="text-xs font-semibold text-brand-400 mb-2 uppercase tracking-wide">Most Popular</div>
      )}
      <h3 className="text-xl font-bold text-dark-100">{name}</h3>
      <div className="mt-4 flex items-baseline gap-1">
        <span className="text-4xl font-bold text-dark-50">{price}</span>
        {period && <span className="text-dark-400 text-sm">/{period}</span>}
      </div>
      <p className="mt-3 text-dark-300 text-sm">{description}</p>
      <ul className="mt-6 space-y-3 flex-1">
        {features.map((f, i) => (
          <li key={i} className="flex items-start gap-2 text-sm text-dark-200">
            <svg className="w-5 h-5 text-brand-500 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            {f}
          </li>
        ))}
      </ul>
      <a href={ctaLink} className={`mt-8 text-center block ${highlighted ? 'btn-primary' : 'btn-secondary'}`}>
        {cta}
      </a>
    </div>
  );
}
