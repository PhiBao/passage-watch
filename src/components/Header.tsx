import { useEffect, useState } from 'react'
import type { Instrument } from '../types'

interface Props {
  instruments: Instrument[]
  marketOpen: boolean | null
}

function useUtcClock(): string {
  const [now, setNow] = useState(() => new Date())
  useEffect(() => {
    const t = setInterval(() => setNow(new Date()), 1000)
    return () => clearInterval(t)
  }, [])
  const p = (n: number) => String(n).padStart(2, '0')
  return `${p(now.getUTCHours())}:${p(now.getUTCMinutes())}:${p(now.getUTCSeconds())} UTC`
}

/**
 * Status bar + ticker tape. The tape is catalog data at a glance: every
 * instrument with a live observed price, scrolling like a quote board.
 * It is aria-hidden decoration; the numbers are real CMC observations.
 */
export default function Header({ instruments, marketOpen }: Props) {
  const clock = useUtcClock()
  const tape = instruments.filter((i) => i.observed_price_usd != null).slice(0, 24)
  const badge = marketOpen == null ? 'standby' : marketOpen ? 'open' : 'closed'

  return (
    <>
      <header className="site-header">
        <div className="brand">
          <svg className="brand-mark" viewBox="0 0 32 32" aria-hidden="true">
            <path d="M6 16h8M14 16c4 0 4-8 12-8M14 16c4 0 4 8 12 8" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" />
            <circle cx="6" cy="16" r="2.6" fill="var(--green)" />
            <circle cx="26" cy="8" r="2.6" fill="var(--green)" />
            <circle cx="26" cy="24" r="2.6" fill="var(--green)" />
          </svg>
          <div>
            <div className="brand-name">PASSAGE WATCH<span className="brand-cursor" aria-hidden="true">▊</span></div>
            <div className="brand-tag">Choose what your tokenized holding becomes.</div>
          </div>
        </div>
        <nav className="site-nav" aria-label="Primary">
          <a href="#method">Method</a>
          <a href="#sources">Sources</a>
        </nav>
        <div className="status-cluster">
          <span className={`market-badge ${badge}`}>
            NYSE · {badge === 'open' ? 'OPEN' : badge === 'closed' ? 'CLOSED' : 'STANDBY'}
          </span>
          <span className="status-clock">{clock}</span>
        </div>
      </header>

      {tape.length > 1 && (
        <div className="ticker-tape" aria-hidden="true">
          <div className="tape-track">
            {[0, 1].map((copy) => (
              <div className="tape-run" key={copy}>
                {tape.map((i) => (
                  <span className="tape-item" key={`${copy}-${i.crypto_id}`}>
                    {i.token_symbol} <b>${i.observed_price_usd!.toFixed(2)}</b>
                  </span>
                ))}
              </div>
            ))}
          </div>
        </div>
      )}
    </>
  )
}
