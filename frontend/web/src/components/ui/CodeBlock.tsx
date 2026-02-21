'use client';

import { useState } from 'react';

interface CodeBlockProps {
  code: string;
  language?: string;
  title?: string;
}

export default function CodeBlock({ code, language = 'bash', title }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="relative group">
      {title && (
        <div className="bg-dark-700 px-4 py-2 rounded-t-lg border border-dark-600/30 border-b-0">
          <span className="text-xs text-dark-300 font-mono">{title}</span>
        </div>
      )}
      <div className={`code-block ${title ? 'rounded-t-none' : ''}`}>
        <button
          onClick={handleCopy}
          className="absolute top-3 right-3 opacity-0 group-hover:opacity-100 transition-opacity
                     text-dark-400 hover:text-dark-100 text-xs px-2 py-1 rounded bg-dark-700"
        >
          {copied ? 'Copied!' : 'Copy'}
        </button>
        <pre><code className={`language-${language}`}>{code}</code></pre>
      </div>
    </div>
  );
}
