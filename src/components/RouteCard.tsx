import type { EvidenceState, RouteResult } from '../types'

const stateLabel: Record<EvidenceState, string> = {
  quoted: 'Quoted',
  terms_only: 'Terms only',
  requires_preparation: 'Requires preparation',
  different_destination: 'Different destination',
  unknown_applicability: 'Unknown applicability',
  conflicting_evidence: 'Conflicting evidence',
  no_quote: 'No quote from this provider',
  stale: 'Stale',
}

export default function RouteCard({ route }: { route: RouteResult }) {
  return (
    <article className={`route-card ${route.applicable ? '' : 'muted'}`}>
      <div className="route-head">
        <h3>{route.title}</h3>
        <span className={`state state-${route.evidence_state}`}>{stateLabel[route.evidence_state]}</span>
      </div>
      <p className="route-outcome">{route.outcome}</p>
      {route.when && <p className="route-when">Available: {route.when}</p>}

      {route.prerequisites.length > 0 && (
        <div className="route-block">
          <h4>Prerequisites</h4>
          <ul>{route.prerequisites.map((p, i) => <li key={i}>{p}</li>)}</ul>
        </div>
      )}

      {route.unknowns.length > 0 && (
        <div className="route-block">
          <h4>Unknowns</h4>
          <ul className="unknowns">{route.unknowns.map((u, i) => <li key={i}>{u}</li>)}</ul>
        </div>
      )}

      {route.source_urls.length > 0 && (
        <details className="route-sources">
          <summary>Sources ({route.source_urls.length})</summary>
          <ul>
            {route.source_urls.map((url, i) => (
              <li key={i}><a href={url} target="_blank" rel="noreferrer">{url}</a></li>
            ))}
          </ul>
        </details>
      )}
    </article>
  )
}
