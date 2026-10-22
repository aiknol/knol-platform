'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';

export const ONBOARDING_KEY = 'cortex_onboarded';

export const STEPS = [
  {
    title: 'See it work',
    text: 'The live demo shows Cortex learning facts and building a knowledge graph as you chat. No signup.',
    href: '/demo/',
    cta: 'Open the demo',
  },
  {
    title: 'Give your AI tools memory',
    text: 'Cortex Local adds persistent memory to Claude Desktop, Cursor and Claude Code with one npm install.',
    href: '/cortex-local/',
    cta: 'Set up Cortex Local',
  },
  {
    title: 'Build with it',
    text: 'Self-host Cortex OSS with Docker, then use the Python or TypeScript SDK. The docs cover the API.',
    href: '/docs/',
    cta: 'Read the docs',
  },
] as const;

function hasOnboarded(): boolean {
  try {
    return localStorage.getItem(ONBOARDING_KEY) === '1';
  } catch {
    return false;
  }
}

function markOnboarded(): void {
  try {
    localStorage.setItem(ONBOARDING_KEY, '1');
  } catch {
    // storage unavailable (private mode) — the guide just shows again next visit
  }
}

export default function OnboardingGuide() {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const [step, setStep] = useState(0);

  useEffect(() => {
    if (pathname === '/' && !hasOnboarded()) setOpen(true);
  }, [pathname]);

  const close = useCallback(() => {
    markOnboarded();
    setOpen(false);
  }, []);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, close]);

  if (!open) return null;

  const last = step === STEPS.length - 1;
  const current = STEPS[step];

  return (
    <div
      role="dialog"
      aria-modal="false"
      aria-labelledby="onboarding-title"
      className="fixed inset-x-4 bottom-4 z-50 mx-auto max-w-md rounded-xl border border-dark-600/50 bg-dark-800 p-5 pt-3 shadow-2xl"
    >
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold text-dark-400">
          New here? Step {step + 1} of {STEPS.length}
        </span>
        <button
          type="button"
          onClick={close}
          aria-label="Close guide"
          className="-mr-3 inline-flex min-h-[48px] min-w-[48px] items-center justify-center text-xl text-dark-400 hover:text-dark-100"
        >
          ×
        </button>
      </div>
      <h2 id="onboarding-title" className="text-lg font-semibold text-dark-50">{current.title}</h2>
      <p className="mt-1 text-sm leading-relaxed text-dark-300">{current.text}</p>
      <Link
        href={current.href}
        onClick={close}
        className="mt-2 inline-flex min-h-[48px] items-center text-sm font-medium text-brand-400 hover:text-brand-300"
      >
        {current.cta} &rarr;
      </Link>
      <div className="mt-2 flex flex-wrap justify-end gap-2">
        <button type="button" onClick={close} className="min-h-[48px] px-4 text-sm text-dark-400 hover:text-dark-100">
          Skip
        </button>
        {step > 0 && (
          <button type="button" onClick={() => setStep(step - 1)} className="btn-secondary min-h-[48px] !py-0 text-sm">
            Back
          </button>
        )}
        <button
          type="button"
          onClick={() => (last ? close() : setStep(step + 1))}
          className="btn-primary min-h-[48px] !py-0 text-sm"
        >
          {last ? 'Done' : 'Next'}
        </button>
      </div>
    </div>
  );
}
