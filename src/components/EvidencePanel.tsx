import type { CheckResult, PlanRecord, PlanChange } from '../types'

function changesFor(record: PlanRecord | null, current: CheckResult | null, price: number | null, session: string): PlanChange[] {
  if (!record || !current) return []
  const out: PlanChange[] = []
  const before = record.observation.observed_price_usd
  if (before != null && price != null && Math.abs(before - price) > 0.0001) {
    const bps = ((price / before) - 1) * 10000
    out.push({ field: 'Observed price', before: `$${before.toFixed(2)}`, after: `$${price.toFixed(2)} (${bps >= 0 ? '+' : ''}${bps.toFixed(1)} bps)` })
  }
  if (record.observation.market_session !== session) {
    out.push({ field: 'Market session', before: record.observation.market_session, after: session })
  }
  const qBefore = record.observation.quote_amount
  const qAfter = current.quote?.output_amount ?? null
  if (qBefore != null && qAfter != null && qBefore !== qAfter) {
    out.push({ field: 'Secondary quote', before: String(qBefore), after: String(qAfter) })
  } else if (qBefore != null && qAfter == null) {
    out.push({ field: 'Secondary quote', before: String(qBefore), after: 'no quote from this provider' })
  }
  return out
}

export default function EvidencePanel({ result, method, savedPlan }: {
  result: CheckResult | null
  method: { endpoints: string[]; rule_version: string } | null
  savedPlan: PlanRecord | null
}) {
  const price = result?.observed_value_usd ?? null
  const session = result ? (new Date(result.checked_at).getUTCHours() >= 13 && new Date(result.checked_at).getUTCHours() < 20 ? 'US market open' : 'US market closed') : '—'
  const changes = changesFor(savedPlan, result, price, session)

  return (
    <section className="evidence-panel" id="sources">
      <div className="evidence-grid">
        <div>
          <h3>Source layers</h3>
          <ul className="source-layers">
            <li><strong>Observed market data</strong> — CoinMarketCap identity, price and timestamps.</li>
            <li><strong>Provider quote</strong> — a size-specific estimate, never a fill.</li>
            <li><strong>Documented route terms</strong> — published process and conditions, not verified personal access.</li>
          </ul>
        </div>
        <div>
          <h3>API evidence</h3>
          {method ? (
            <>
              <p className="field-hint">Rule version <code>{method.rule_version}</code></p>
              <ul className="endpoint-list">
                {method.endpoints.map((e, i) => <li key={i}><code>{e}</code></li>)}
              </ul>
            </>
          ) : <p className="field-hint">Load a check to see the endpoints used.</p>}
        </div>
        <div>
          <h3>Change since saved</h3>
          {savedPlan ? (
            changes.length ? (
              <ul className="change-list">
                {changes.map((c, i) => (
                  <li key={i}><strong>{c.field}:</strong> {c.before} → {c.after}</li>
                ))}
              </ul>
            ) : <p className="field-hint">Nothing changed since this plan was saved.</p>
          ) : <p className="field-hint">Save a plan, then re-check to see what changed.</p>}
        </div>
      </div>
    </section>
  )
}
