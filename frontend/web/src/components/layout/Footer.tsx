import Link from 'next/link';
import { SITE, FOOTER_SECTIONS } from '@/config/site';

export default function Footer() {
  return (
    <footer className="border-t border-dark-600/30 mt-20">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <nav aria-label="Footer navigation">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
            {FOOTER_SECTIONS.map((section) => (
            <div key={section.title}>
              <h3 className="text-sm font-semibold text-dark-100 mb-4">{section.title}</h3>
              <ul className="space-y-2">
                {section.links.map((link) => {
                  const isExternal = 'external' in link && link.external;
                  const href = String(link.href);
                  const isProtocol = href.startsWith('mailto:') || href.startsWith('tel:') || href.startsWith('http://') || href.startsWith('https://');
                  const isAbsoluteWeb = href.startsWith('http://') || href.startsWith('https://');
                  const className = "text-dark-300 hover:text-brand-500 text-sm";
                  return (
                    <li key={String(link.label)}>
                      {isExternal || isProtocol ? (
                        <a
                          href={href}
                          {...(isExternal || isAbsoluteWeb ? { target: "_blank", rel: "noopener noreferrer" } : {})}
                          className={className}
                        >
                          {link.label}
                        </a>
                      ) : (
                        <Link href={href} className={className}>{link.label}</Link>
                      )}
                    </li>
                  );
                })}
              </ul>
            </div>
            ))}
          </div>
        </nav>
        <div className="mt-8 pt-8 border-t border-dark-600/30 text-center">
          <p className="text-dark-400 text-sm">&copy; {new Date().getFullYear()} {SITE.name}. All rights reserved.</p>
        </div>
      </div>
    </footer>
  );
}
