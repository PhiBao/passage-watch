import { useEffect, useState } from 'react'
import Header from './components/Header'
import PlanForm from './components/PlanForm'
import RouteCard from './components/RouteCard'
import EvidencePanel from './components/EvidencePanel'
import { fetchCatalog, fetchMethod, runCheck } from './api'
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
      const res = await runCheck(input)
      setResult(res)
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
        quote_amount: result.quote?.output_amount ?? null,
        market_session: new Date(result.checked_at).getUTCHours() >= 13 && new Date(result.checked_at).getUTCHours() < 20 ? 'US market open' : 'US market closed',
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
                  <span className="quote-label">Secondary quote (read-only)</span>
                  {result.quote.error
                    ? <span className="quote-value warn">No quote from this provider</span>
                    : <span className="quote-value">{result.quote.output_amount} {result.quote.output_label}</span>}
                </div>
                <p className="field-hint">
                  Provider {result.quote.provider} · retrieved {new Date(result.quote.retrieved_at).toLocaleTimeString()}
                  {result.quote.expires_at && ` · expires ${new Date(result.quote.expires_at).toLocaleTimeString()}`}
                  . Estimate only — never a guaranteed fill.
                </p>
              </div>
            )}

            <div className="route-grid">
              {result.routes.map((route) => <RouteCard key={route.kind} route={route} />)}
            </div>
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
