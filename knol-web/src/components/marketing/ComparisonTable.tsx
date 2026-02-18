import { COMPARISON_FEATURES } from '@/config';

function Check() {
  return (
    <svg className="w-5 h-5 text-emerald-400 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-label="Feature supported" role="img">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
    </svg>
  );
}

function Cross() {
  return <span className="text-dark-500 text-lg" aria-label="Feature not supported" role="img">&mdash;</span>;
}

export default function ComparisonTable() {
  return (
    <div className="overflow-x-auto rounded-xl border border-dark-600/30">
      <table className="w-full text-sm min-w-[600px]">
        <thead>
          <tr className="bg-dark-800/80">
            <th className="text-left py-3 px-4 text-dark-300 font-medium">Feature</th>
            <th className="text-center py-3 px-4 text-dark-300 font-medium w-24">Mem0</th>
            <th className="text-center py-3 px-4 text-dark-300 font-medium w-24">Zep</th>
            <th className="text-center py-3 px-4 font-semibold text-brand-400 w-24">Knol</th>
          </tr>
        </thead>
        <tbody>
          {COMPARISON_FEATURES.map(({ feature, mem0, zep, knol }) => (
            <tr key={feature} className="border-t border-dark-600/20 hover:bg-dark-800/30 transition-colors">
              <td className="py-3 px-4 text-dark-200">{feature}</td>
              <td className="text-center py-3 px-4">{mem0 ? <Check /> : <Cross />}</td>
              <td className="text-center py-3 px-4">{zep ? <Check /> : <Cross />}</td>
              <td className="text-center py-3 px-4 bg-brand-500/5">{knol ? <Check /> : <Cross />}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
