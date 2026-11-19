import { useEffect, useState } from 'react'
import Header from './components/Header'
import PlanForm from './components/PlanForm'
import RouteCard from './components/RouteCard'
import TimelinePanel from './components/TimelinePanel'
import EvidencePanel from './components/EvidencePanel'
import { fetchCatalog, fetchChanges, fetchMethod, fetchObservations, fetchTimeline, runCheck } from './api'
import { isUsdstHolidayDate, marketSessionFromCheck } from './session'
import type { ObservationPoint, Timeline, ChangeEntry } from './api'
import type {
  CheckResult,
  Destination,
  Instrument,
  Onboarding,
  PlanInput,
  PlanRecord,
} from './types'

const STORAGE_KEY = 'passage-watch-plan-v1'

function loadSaved(): PlanRecord | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? (JSON.parse(raw) as PlanRecord) : null
  } catch {
    return null
  }
}

export default function App() {
  const [instruments, setInstruments] = useState<Instrument[]>([])
  const [selectedId, setSelectedId] = useState<number | null>(null)
  const [quantity, setQuantity] = useState('75')
  const [destination, setDestination] = useState<Destination>('USDC_SOLANA')
  const [deadline, setDeadline] = useState('')
  const [issuerOnboarding, setIssuerOnboarding] = useState<Onboarding>('unknown')
  const [brokerOnboarding, setBrokerOnboarding] = useState<Onboarding>('unknown')
  const [result, setResult] = useState<CheckResult | null>(null)
  const [method, setMethod] = useState<{ endpoints: string[]; rule_version: string } | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [savedPlan, setSavedPlan] = useState<PlanRecord | null>(loadSaved)
  const [observations, setObservations] = useState<ObservationPoint[]>([])
  const [timeline, setTimeline] = useState<Timeline | null>(null)
  const [changes, setChanges] = useState<ChangeEntry[]>([])
  const selectedInstrument = instruments.find((i) => i.crypto_id === selectedId)

  useEffect(() => {
    fetchCatalog()
      .then((data) => setInstruments(data.instruments))
      .catch((e: Error) => setError(`Could not load instrument catalog: ${e.message}`))
    fetchMethod().then(setMethod).catch(() => undefined)
  }, [])

  function buildInput(): PlanInput | null {
    if (selectedId == null || !quantity || Number(quantity) <= 0) return null
    return {
      crypto_id: selectedId,
      quantity: Number(quantity),
      destination,
      deadline: deadline || new Date().toISOString().slice(0, 10),
      issuer_onboarding: issuerOnboarding,
      broker_onboarding: brokerOnboarding,
    }
  }

  async function onCheck() {
    const input = buildInput()
    if (!input) {
      setError('Choose an instrument and enter a quantity above zero.')
      return
    }
    setLoading(true)
    setError(null)
    try {
      const [res, obs] = await Promise.all([runCheck(input), fetchObservations(input.crypto_id)])
      setResult(res)
      setObservations(obs.points.filter((p) => p.price_usd != null))

      // Exit-window timeline + change history (best-effort; read-only).
      const tokenSymbol = selectedInstrument?.token_symbol
      if (tokenSymbol) {
        const deadline = input.deadline || new Date().toISOString().slice(0, 10)
        const [tl, ch] = await Promise.all([
          fetchTimeline(tokenSymbol, deadline, input.destination).catch(() => null),
          fetchChanges(tokenSymbol, savedPlan?.saved_at).catch(() => null),
        ])
        setTimeline(tl)
        setChanges(ch?.changes ?? [])
      }
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  function onSave() {
    if (!result) return
    const record: PlanRecord = {
      input: buildInput()!,
      observation: {
        checked_at: result.checked_at,
        observed_price_usd: result.observed_value_usd,
        quote_amount: result.quote?.usdc_amount ?? null,
        market_session: isUsdstHolidayDate(result.checked_at.slice(0, 10)) ? 'US market closed' : marketSessionFromCheck(result.checked_at),
        evidence_reviewed: result.rule_version,
      },
      saved_at: new Date().toISOString(),
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(record))
    setSavedPlan(record)
  }

  function onClear() {
    localStorage.removeItem(STORAGE_KEY)
    setSavedPlan(null)
  }

  return (
    <div className="app-shell">
      <Header />
      <main>
        <section className="hero">
          <p className="eyebrow">Outcome planner for tokenized holdings</p>
          <h1>Same holding.<br />Different destination.<br /><em>Different route.</em></h1>
          <p className="hero-sub">
            Choose an exact tokenized instrument and what you need it to become.
            Passage Watch shows the relevant routes, their prerequisites and their unknowns —
            then tells you what changed since you saved the plan.
          </p>
          <p className="hero-note">Read-only. No wallet signing, no execution, no legal eligibility verdict.</p>
        </section>

        <section className="workbench">
          <h2 className="section-title">Plan an exit</h2>
          <PlanForm
            instruments={instruments}
            selectedId={selectedId}
            onSelect={setSelectedId}
            quantity={quantity}
            onQuantity={setQuantity}
            destination={destination}
            onDestination={setDestination}
            deadline={deadline}
            onDeadline={setDeadline}
            issuerOnboarding={issuerOnboarding}
            onIssuer={setIssuerOnboarding}
            brokerOnboarding={brokerOnboarding}
            onBroker={setBrokerOnboarding}
            onCheck={onCheck}
            loading={loading}
            error={error}
          />
        </section>

        {result && (
          <section className="results">
            <div className="results-head">
              <div>
                <h2 className="section-title">{result.instrument.token_symbol} → {result.destination === 'USDC_SOLANA' ? 'USDC' : 'brokerage shares'}</h2>
                <p className="field-hint">
                  {result.quantity} units · checked {new Date(result.checked_at).toLocaleString()}
                  {result.observed_value_usd != null && ` · observed value $${result.observed_value_usd.toFixed(2)}`}
                </p>
              </div>
              <div className="save-actions">
                <button className="secondary-action" onClick={onSave}>Watch this plan</button>
                {savedPlan && <button className="text-action" onClick={onClear}>Clear saved plan</button>}
              </div>
            </div>

            {result.quote && (
              <div className="quote-strip">
                <div>
                  <span className="quote-label">
                    Sell {result.quantity} {result.instrument.token_symbol} → USDC
                  </span>
                  {result.quote.error || result.quote.usdc_amount == null
                    ? <span className="quote-value warn">No quote from this provider</span>
                    : <span className="quote-value">${result.quote.usdc_amount.toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>}
                </div>
                <p className="field-hint">
                  Jupiter Swap V2 (read-only) · retrieved {new Date(result.quote.retrieved_at).toLocaleTimeString()}
                  {result.quote.spread_vs_cmc_bps != null && result.quote.spread_vs_cmc_bps !== 0 && result.quote.spread_vs_cmc_bps > 0
                    ? ` · quotes +${result.quote.spread_vs_cmc_bps.toFixed(0)} bps above CMC`
                    : result.quote.spread_vs_cmc_bps != null && result.quote.spread_vs_cmc_bps !== 0
                      ? ` · quotes ${result.quote.spread_vs_cmc_bps.toFixed(0)} bps below CMC`
                      : ''}
                  . Estimate only — never a guaranteed fill.
                </p>
              </div>
            )}

            {observations.length > 1 && (
              <div className="observation-strip">
                <div className="observation-head">
                  <span className="quote-label">Recorded observations · last {observations.length} days</span>
                  <span className="observation-change">
                    {(() => {
                      const first = observations[0].price_usd
                      const last = observations[observations.length - 1].price_usd
                      const bps = ((last / first) - 1) * 10000
                      return `${bps >= 0 ? '+' : ''}${bps.toFixed(0)} bps over the window`
                    })()}
                  </span>
                </div>
                <div className="observation-bars" aria-hidden="true">
                  {observations.map((p, i) => {
                    const min = Math.min(...observations.map((o) => o.price_usd))
                    const max = Math.max(...observations.map((o) => o.price_usd))
                    const h = 12 + ((p.price_usd - min) / (max - min || 1)) * 30
                    return <span key={i} className="obs-bar" style={{ height: `${h}px` }} title={`${p.timestamp.slice(0, 10)} · $${p.price_usd.toFixed(2)}`} />
                  })}
                </div>
                <p className="field-hint">Real daily CMC quotes. This is the evidence behind “change since saved”.</p>
              </div>
            )}

            <div className="route-grid">
              {result.routes.map((route) => <RouteCard key={route.kind} route={route} />)}
            </div>

            <TimelinePanel timeline={timeline} changes={changes} />
          </section>
        )}

        <EvidencePanel result={result} method={method} savedPlan={savedPlan} />
      </main>

      <section className="method" id="method">
        <h2 className="section-title">How this works</h2>
        <div className="method-grid">
          <div><h3>Identity</h3><p>CoinMarketCap resolves the underlying asset, its wrapper tokens and issuers. Every plan is pinned to that identity.</p></div>
          <div><h3>Routes</h3><p>Published issuer routes and a read-only secondary quote are compared against your destination. Different routes deliver different end states.</p></div>
          <div><h3>Honesty</h3><p>Quotes are not fills. Shares are not cash. Self-reported onboarding is never verified eligibility. Unknowns stay visible.</p></div>
        </div>
      </section>

      <footer className="site-footer">
        <span>Passage Watch · data by CoinMarketCap · read-only prototype</span>
        <span>Not financial, legal or tax advice.</span>
      </footer>
    </div>
  )
}
