import type { Destination, Instrument, Onboarding } from '../types'

interface Props {
  instruments: Instrument[]
  selectedId: number | null
  onSelect: (id: number) => void
  quantity: string
  onQuantity: (value: string) => void
  destination: Destination
  onDestination: (value: Destination) => void
  deadline: string
  onDeadline: (value: string) => void
  issuerOnboarding: Onboarding
  onIssuer: (value: Onboarding) => void
  brokerOnboarding: Onboarding
  onBroker: (value: Onboarding) => void
  onCheck: () => void
  loading: boolean
  error: string | null
}

function OnboardingChoice({ label, value, onChange }: {
  label: string
  value: Onboarding
  onChange: (v: Onboarding) => void
}) {
  return (
    <fieldset className="onboarding">
      <legend>{label}</legend>
      <div className="segmented">
        {(['unknown', 'met', 'unmet'] as Onboarding[]).map((opt) => (
          <label key={opt} className={value === opt ? 'seg active' : 'seg'}>
            <input
              type="radio"
              name={label}
              value={opt}
              checked={value === opt}
              onChange={() => onChange(opt)}
            />
            <span>{opt}</span>
          </label>
        ))}
      </div>
      <p className="field-hint">Self-reported only. This is never a verified eligibility result.</p>
    </fieldset>
  )
}

export default function PlanForm(props: Props) {
  const selected = props.instruments.find((i) => i.crypto_id === props.selectedId)

  return (
    <form className="plan-form" onSubmit={(e) => { e.preventDefault(); props.onCheck() }}>
      <div className="field">
        <label htmlFor="instrument">1. What do you hold?</label>
        <select
          id="instrument"
          value={props.selectedId ?? ''}
          onChange={(e) => props.onSelect(Number(e.target.value))}
        >
          <option value="" disabled>Select an exact tokenized instrument</option>
          {props.instruments.map((i) => (
            <option key={i.crypto_id} value={i.crypto_id}>
              {i.token_symbol} — {i.underlying_name} ({i.network})
            </option>
          ))}
        </select>
        {selected && (
          <p className="identity-line">
            {selected.issuer} · contract <code>{selected.contract.slice(0, 8)}…{selected.contract.slice(-6)}</code>
            {selected.unit_basis === 'verified' ? ' · unit verified' : ' · unit unknown'}
          </p>
        )}
      </div>

      <div className="field-row">
        <div className="field">
          <label htmlFor="quantity">2. How much?</label>
          <input
            id="quantity"
            type="number"
            min="0"
            step="any"
            inputMode="decimal"
            placeholder="e.g. 75"
            value={props.quantity}
            onChange={(e) => props.onQuantity(e.target.value)}
          />
          <p className="field-hint">Displayed token units, not raw on-chain units.</p>
        </div>
        <div className="field">
          <label htmlFor="deadline">3. By when?</label>
          <input
            id="deadline"
            type="date"
            value={props.deadline}
            onChange={(e) => props.onDeadline(e.target.value)}
          />
        </div>
      </div>

      <fieldset className="field">
        <legend>4. What should it become?</legend>
        <div className="destination-grid">
          <label className={props.destination === 'USDC_SOLANA' ? 'destination active' : 'destination'}>
            <input
              type="radio"
              name="destination"
              value="USDC_SOLANA"
              checked={props.destination === 'USDC_SOLANA'}
              onChange={() => props.onDestination('USDC_SOLANA')}
            />
            <span className="destination-title">USDC in my Solana wallet</span>
            <span className="destination-sub">Stablecoin liquidity</span>
          </label>
          <label className={props.destination === 'BROKERAGE_SHARES' ? 'destination active' : 'destination'}>
            <input
              type="radio"
              name="destination"
              value="BROKERAGE_SHARES"
              checked={props.destination === 'BROKERAGE_SHARES'}
              onChange={() => props.onDestination('BROKERAGE_SHARES')}
            />
            <span className="destination-title">Shares in my brokerage</span>
            <span className="destination-sub">Keep the equity exposure</span>
          </label>
        </div>
      </fieldset>

      <div className="field-row">
        <OnboardingChoice
          label="Issuer onboarding"
          value={props.issuerOnboarding}
          onChange={props.onIssuer}
        />
        <OnboardingChoice
          label="Brokerage onboarding"
          value={props.brokerOnboarding}
          onChange={props.onBroker}
        />
      </div>

      {props.error && <p className="error-banner" role="alert">{props.error}</p>}

      <button className="primary-action" type="submit" disabled={props.loading || !selected}>
        {props.loading ? 'Checking routes…' : 'Check routes'}
      </button>
    </form>
  )
}
