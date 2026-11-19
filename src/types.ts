export type Destination = 'USDC_SOLANA' | 'BROKERAGE_SHARES'
export type Onboarding = 'met' | 'unmet' | 'unknown'

export interface Instrument {
  underlying: string
  underlying_name: string
  rwa_id: number
  token_symbol: string
  crypto_id: number
  issuer: string
  network: string
  contract: string
  observed_price_usd: number | null
  unit_basis: 'verified' | 'unknown'
}

export type RouteKind = 'SECONDARY_SALE' | 'MARKET_FLOW' | 'XCHANGE' | 'XPORT'

export type EvidenceState =
  | 'quoted'
  | 'terms_only'
  | 'requires_preparation'
  | 'different_destination'
  | 'unknown_applicability'
  | 'conflicting_evidence'
  | 'no_quote'
  | 'stale'

export interface RouteResult {
  kind: RouteKind
  title: string
  outcome: string
  applicable: boolean
  evidence_state: EvidenceState
  prerequisites: string[]
  unknowns: string[]
  source_urls: string[]
  when?: string | null
}

export interface SellQuote {
  token_symbol: string
  quantity_displayed: number
  raw_token_in: number
  usdc_amount: number | null
  output_label: string
  spread_vs_cmc_bps: number | null
  usdc_per_share_estimate: number | null
  retrieved_at: string
  error: string | null
  is_ready_to_confirm: boolean
}

export interface SizeQuote {
  fraction: number
  quantity: number
  usdc_amount: number | null
  usdc_per_share: number | null
  spread_vs_cmc_bps: number | null
  error: string | null
}

export interface CheckResult {
  instrument: Instrument
  destination: Destination
  quantity: number
  deadline: string
  observed_value_usd: number | null
  quote: SellQuote | null
  size_quotes: SizeQuote[]
  routes: RouteResult[]
  checked_at: string
  rule_version: string
}

export interface PlanInput {
  crypto_id: number
  quantity: number
  destination: Destination
  deadline: string
  issuer_onboarding: Onboarding
  broker_onboarding: Onboarding
}

export interface ObservationSnapshot {
  checked_at: string
  observed_price_usd: number | null
  quote_amount: number | null
  market_session: string
  evidence_reviewed: string
}

export interface PlanRecord {
  input: PlanInput
  observation: ObservationSnapshot
  saved_at: string
}

export interface PlanChange {
  field: string
  before: string
  after: string
}
