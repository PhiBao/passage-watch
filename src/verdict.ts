import type { CheckResult } from './types'

export function verdict(result: CheckResult | null): string | null {
  if (!result) return null
  const { quote, routes, destination } = result
  if (!quote) {
    return `Enter a quantity and check routes to see the sell-side quote and route statuses.`
  }
  if (quote.error || quote.usdc_amount == null) {
    return `No live quote right now from the secondary quote provider.`
  }

  const needsPrep = routes.filter((r) => r.applicable && r.evidence_state === 'requires_preparation').length
  const different = routes.find((r) => r.applicable && r.evidence_state === 'different_destination')

  const base = destination === 'USDC_SOLANA'
    ? `Sell ${result.quantity} ${result.instrument.token_symbol} → USDC ≈ $${quote.usdc_amount.toLocaleString(undefined, { minimumFractionDigits: 2 })}.`
    : `For brokerage shares, look at xPort (prereqs below) rather than cash — shares are not cash received.`

  let add = ''
  if (different) {
    add = ` This destination needs ${different.title} to be a different route.`
  }
  if (needsPrep > 0) {
    add += ` ${needsPrep} applicable route${needsPrep > 1 ? 's' : ''} need preparation before you act.`
  }
  if (result.observed_value_usd != null) {
    const bps = quote.spread_vs_cmc_bps
    if (bps && bps !== 0) {
      add += ` Jupiter quotes ${bps >= 0 ? '+' : ''}${bps.toFixed(0)} bps ${bps >= 0 ? 'above' : 'below'} the CMC observed notional.`
    }
  }
  return base + add
}

export function deadlineHint(iso: string | undefined): string | null {
  if (!iso) return null
  const d = new Date(iso)
  if (isNaN(d.getTime())) return null
  return `${d.toISOString().slice(0, 10)} (${d.toISOString().slice(11, 16)} UTC)`
}
