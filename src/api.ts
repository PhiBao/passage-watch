import type {
  CheckResult,
  Instrument,
  PlanInput,
} from './types'

/**
 * The browser only ever talks to the same-origin product API.
 * The CMC key and quote-provider credentials stay server-side.
 */
async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(path, { headers: { Accept: 'application/json' } })
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
