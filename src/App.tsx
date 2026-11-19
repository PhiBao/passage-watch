import { useEffect, useState } from 'react'
import Header from './components/Header'
import PlanForm from './components/PlanForm'
import RouteCard from './components/RouteCard'
import TimelinePanel from './components/TimelinePanel'
import Sparkline from './components/Sparkline'
import EvidencePanel from './components/EvidencePanel'
import { fetchAddressHoldings, fetchCatalog, fetchChanges, fetchMethod, fetchObservations, fetchTimeline, runCheck } from './api'
import type { Holding } from './api'
import { isUsdstHolidayDate, marketSessionFromCheck } from './session'
import type { ObservationPoint, Timeline, ChangeEntry } from './api'
import { verdict as computeVerdict } from './verdict'
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
  const [deadline, setDeadline] = useState(() => {
    const d = new Date(Date.now() + 3 * 86400000)
    return d.toISOString().slice(0, 10)
  })
  const [issuerOnboarding, setIssuerOnboarding] = useState<Onboarding>('unknown')
  const [brokerOnboarding, setBrokerOnboarding] = useState<Onboarding>('unknown')
  const [result, setResult] = useState<CheckResult | null>(null)
  const [verdictText, setVerdictText] = useState<string | null>(null)
  const [method, setMethod] = useState<{ endpoints: string[]; rule_version: string } | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [savedPlan, setSavedPlan] = useState<PlanRecord | null>(loadSaved)
  const [observations, setObservations] = useState<ObservationPoint[]>([])
  const [timeline, setTimeline] = useState<Timeline | null>(null)
  const [changes, setChanges] = useState<ChangeEntry[]>([])
  const [addressInput, setAddressInput] = useState('')
  const [importStatus, setImportStatus] = useState('')
  const [holdings, setHoldings] = useState<Holding[]>([])

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
      setVerdictText(computeVerdict(res))
      setObservations(obs.points.filter((p) => p.price_usd != null))

      // Exit-window timeline + change history (best-effort; read-only).
      // Look the symbol up from the submitted input, not from a render-time
      // closure: a fast select-then-check would otherwise skip the timeline.
      const tokenSymbol = instruments.find((i) => i.crypto_id === input.crypto_id)?.token_symbol
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

  async function onImportAddress() {
    const addr = addressInput.trim()
    if (!addr) return
    setImportStatus('Scanning address…')
    setHoldings([])
    try {
      const res = await fetchAddressHoldings(addr)
      const found = res.instruments
      if (found.length === 0) {
        setImportStatus('No supported tokenized-stock holdings found in this address.')
        return
      }
      setHoldings(found)
      const first = found[0]
      const inst = instruments.find((i) => i.token_symbol === first.token_symbol)
      if (inst) {
        setSelectedId(inst.crypto_id)
        setQuantity(String(first.balance))
        setImportStatus(`✓ ${found.length} supported holding${found.length > 1 ? 's' : ''} found — selected ${first.token_symbol}.`)
      } else {
        setImportStatus(`${found.length} holding${found.length > 1 ? 's' : ''} found.`)
      }
    } catch (e) {
      setImportStatus((e as Error).message)
    }
  }

  function chooseHolding(tokenSymbol: string, balance: number) {
    const inst = instruments.find((i) => i.token_symbol === tokenSymbol)
    if (!inst) return
    setSelectedId(inst.crypto_id)
    setQuantity(String(balance))
    setImportStatus(`✓ Selected ${tokenSymbol} (${balance} units).`)
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
          <div className="address-row">
            <label htmlFor="address">Optional: from your wallet (read-only)</label>
            <div className="address-input-group">
              <input
                id="address"
                type="text"
                placeholder="Paste a Solana address to scan holdings"
                value={addressInput}
                onChange={(e) => setAddressInput(e.target.value)}
                onKeyDown={(e) => { if (e.key === 'Enter') onImportAddress() }}
              />
              <button className="secondary-action" onClick={onImportAddress}>Scan holdings</button>
            </div>
            {importStatus && <span className={`import-status ${/✓|No supported|prefer/.test(importStatus) ? 'ok' : 'warn'}`}>{importStatus}</span>}
            {holdings.length > 0 && (
              <ul className="holdings-list">
                {holdings.map((h) => (
                  <li key={h.contract}>
                    <button className="holding-chip" onClick={() => chooseHolding(h.token_symbol, h.balance)}>
                      <strong>{h.token_symbol}</strong>
                      <span>{h.balance.toLocaleString(undefined, { maximumFractionDigits: 4 })} units</span>
                      <span className="muted">{h.issuer}</span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
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

            {verdictText && (
              <div className="verdict-card">
                <div className="verdict-label">
                  <span>What this plan tells you</span>
                  <span className="verdict-coverage">{instruments.length} instruments in catalog</span>
                </div>
                <p className="verdict-text">{verdictText}</p>
              </div>
            )}

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

            {result.size_quotes && result.size_quotes.length > 0 && (
              <div className="size-strip">
                <span className="quote-label">Liquidity at your size</span>
                <div className="size-grid">
                  {result.size_quotes.map((s) => (
                    <div className="size-cell" key={s.fraction}>
                      <div className="size-head">
                        <span className="size-fraction">{Math.round(s.fraction * 100)}%</span>
                        <span className="size-qty">{s.quantity.toLocaleString(undefined, { maximumFractionDigits: 2 })} units</span>
                      </div>
                      {s.usdc_amount == null
                        ? <span className="size-value warn">no quote</span>
                        : <span className="size-value">${s.usdc_amount.toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>}
                      <span className="size-sub">
                        {s.usdc_per_share != null ? `$${s.usdc_per_share.toFixed(2)}/share` : '—'}
                        {s.spread_vs_cmc_bps != null ? ` · ${s.spread_vs_cmc_bps >= 0 ? '+' : ''}${s.spread_vs_cmc_bps.toFixed(0)} bps` : ''}
                      </span>
                    </div>
                  ))}
                </div>
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
                <Sparkline points={observations} />
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
