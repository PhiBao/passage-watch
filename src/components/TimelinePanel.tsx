import type { ChangeEntry, Timeline } from '../api'
import { fmtHmTime } from '../session'

function fmtIso(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  return `${d.toUTCString().slice(0, 3)}, ${d.toISOString().slice(5, 10)} ${fmtHmTime(d.getUTCHours() * 60 + d.getUTCMinutes())} UTC`
}

function routeChips(routes: Timeline['routes']): string {
  return routes
    .filter((r) => r.applicable)
    .map((r) => {
      const base = `${r.kind} (${r.evidence_state})`
      return r.when ? `${base} · ${r.when}` : base
    })
    .join(', ')
}

export default function TimelinePanel({ timeline, changes }: {
  timeline: Timeline | null
  changes: ChangeEntry[]
}) {
  if (!timeline) return null

  const ns = timeline.next_session
  const changeRows = changes.slice(0, 5).map((c, i) => (
    <div className="change-row" key={i}>
      <div>
        <span className="change-label">{c.price_usd != null ? `Price · $${c.price_usd.toFixed(2)}` : c.usdc_at_monitor_size != null ? `USDC@50 · $${c.usdc_at_monitor_size.toFixed(2)}` : 'change'}</span>
        <span className="change-time">{fmtIso(c.at_iso)}</span>
      </div>
    </div>
  ))

  return (
    <section className="timeline-panel">
      <div className="timeline-head">
        <h3>Exit window</h3>
        <span className={`state state-${timeline.market_open_now ? 'quoted' : 'different_destination'}`}>
          {timeline.market_open_now ? 'Market open now' : 'Market closed now'}
        </span>
      </div>
      <div className="timeline-grid">
        <div>
          <h4>Next session</h4>
          <p>{ns ? `${fmtIso(ns.open)} → ${fmtIso(ns.close)} (UTC)` : 'No further session'}</p>
        </div>
        <div>
          <h4>Deadline</h4>
          <p>{timeline.deadline}</p>
        </div>
        <div>
          <h4>Route relevance</h4>
          <p>{routeChips(timeline.routes) || 'No routes applicable'}</p>
          <p className="deadline-note">
            {timeline.deadline_after_next_session_open
              ? 'The deadline is after the next session opens.'
              : 'The deadline is before the next session opens.'}
          </p>
        </div>
      </div>
      {changes.length > 0 && (
        <div className="change-history">
          <h4>Changes since saved</h4>
          {changeRows}
        </div>
      )}
    </section>
  )
}
