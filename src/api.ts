import type {
  CheckResult,
  Destination,
  Instrument,
  PlanInput,
} from './types'

/**
 * The browser only ever talks to the same-origin product API.
 * The CMC key and quote-provider credentials stay server-side.
 *
 * A scale-to-zero host can drop the very first request while the machine
 * wakes, so a transport failure or a 502/503/504 gets two short retries.
 */
const RETRY_STATUS = new Set([502, 503, 504])

async function getJson<T>(path: string, attempt = 0): Promise<T> {
  let res: Response
  try {
    res = await fetch(path, { headers: { Accept: 'application/json' } })
  } catch (e) {
    if (attempt < 3) {
      await new Promise((r) => setTimeout(r, 1000 + attempt * 800))
      return getJson<T>(path, attempt + 1)
    }
    throw new Error(
      e instanceof TypeError ? 'Service was waking up. Please retry.' : String(e),
    )
  }

  if (RETRY_STATUS.has(res.status) && attempt < 3) {
    await new Promise((r) => setTimeout(r, 1000 + attempt * 800))
    return getJson<T>(path, attempt + 1)
  }

  if (!res.ok) {
    let message = `Request failed (${res.status})`
    try {
      const body = (await res.json()) as { error?: string }
      if (body.error) message = body.error
    } catch {
      // non-JSON error body
    }
    throw new Error(message)
  }
  return (await res.json()) as T
}

export function fetchCatalog(): Promise<{ instruments: Instrument[] }> {
  return getJson('/api/v1/catalog')
}

export function fetchInstrument(cryptoId: number): Promise<Instrument> {
  return getJson(`/api/v1/instruments/${cryptoId}`)
}

export function runCheck(input: PlanInput): Promise<CheckResult> {
  return getJson<CheckResult>(`/api/v1/check?${new URLSearchParams({
    crypto_id: String(input.crypto_id),
    quantity: String(input.quantity),
    destination: input.destination,
    deadline: input.deadline,
    issuer_onboarding: input.issuer_onboarding,
    broker_onboarding: input.broker_onboarding,
  })}`)
}

export function fetchMethod(): Promise<{ endpoints: string[]; rule_version: string }> {
  return getJson('/api/v1/method')
}

export interface ObservationPoint {
  timestamp: string
  price_usd: number
}

export function fetchObservations(cryptoId: number): Promise<{ points: ObservationPoint[] }> {
  return getJson(`/api/v1/observations/${cryptoId}`)
}

export interface ChangeEntry {
  at_iso: string
  price_usd: number | null
  usdc_at_monitor_size: number | null
  quote_error: string | null
}

export function fetchChanges(tokenSymbol: string, sinceIso?: string): Promise<{ token_symbol: string; changes: ChangeEntry[] }> {
  const q = new URLSearchParams()
  if (sinceIso) q.set('since_iso', sinceIso)
  return getJson(`/api/v1/changes/${tokenSymbol}${q.toString() ? `?${q.toString()}` : ''}`)
}

export interface TimelineRoute {
  kind: string
  applicable: boolean
  evidence_state: string
  when?: string | null
}

export interface Timeline {
  token_symbol: string
  checked_at: string
  market_open_now: boolean
  next_session: { open: string; close: string } | null
  deadline: string
  deadline_iso: string
  deadline_after_next_session_open: boolean
  sessions_before_deadline: number
  routes: TimelineRoute[]
}

export function fetchTimeline(tokenSymbol: string, deadline: string, destination: Destination): Promise<Timeline> {
  return getJson(`/api/v1/timeline?${new URLSearchParams({ token_symbol: tokenSymbol, destination, deadline })}`)
}

export interface Holding {
  token_symbol: string
  crypto_id: number
  underlying: string
  balance: number
  contract: string
  issuer: string
}

export function fetchAddressHoldings(address: string): Promise<{ wallet: string; instruments: Holding[] }> {
  return getJson(`/api/v1/address/${address}`)
}
