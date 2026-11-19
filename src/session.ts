// Compact US-market session helpers used by the UI for display of session
// state and exit-window timing. Mirrors the NYSE holiday set in src/calendar.rs.
// All of this is display-only and honest: it powers labels, never trading.

// NYSE holiday dates (UTC-date labels, in US Eastern).
const HOLIDAYS = new Set(
  [
    '2026-01-01', '2026-01-19', '2026-02-16', '2026-04-03', '2026-05-25',
    '2026-06-19', '2026-07-03', '2026-09-07', '2026-11-26', '2026-12-25',
    '2027-01-01', '2027-01-18', '2027-02-15', '2027-04-02', '2027-05-24',
    '2027-06-18', '2027-07-03', '2027-07-04', '2027-09-06', '2027-11-25',
    '2027-12-24', '2027-12-25',
  ],
)

const OPEN = 13.5 // NYSE opens 9:30 AM ET = 13:30 UTC (regular-hours fallback)
const CLOSE = 20.0 // 4:00 PM ET = 20:00 UTC

export function isUsdstHolidayDate(iso: string): boolean {
  return HOLIDAYS.has(iso)
}

function utcMinutes(iso: string): number | null {
  if (!iso) return null
  const m = /^(\d{4})-(\d{2})-(\d{2})[T ]?(\d{2}):(\d{2})/.exec(iso)
  if (!m) return null
  return parseFloat(m[4]) * 60 + parseFloat(m[5])
}

function utcWeekday0(iso: string): number | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso)
  if (!m) return null
  const d = new Date(Date.UTC(+m[1], +m[2] - 1, +m[3]))
  return d.getUTCDay() // 0 = Sunday
}

/** Returns true if a weekday (0=Sun..6=Sat) is inside the regular session window. */
export function sessionWindowOpen(iso: string, minutes: number = utcMinutes(iso) ?? 0): boolean {
  const wd = utcWeekday0(iso)
  if (wd == null) return false
  if (wd === 0 || wd === 6) return false // weekend
  if (HOLIDAYS.has(iso.slice(0, 10))) return false
  return minutes >= OPEN * 60 && minutes < CLOSE * 60
}

/** marketSessionFromCheck: 'US market open' | 'US market closed' */
export function marketSessionFromCheck(iso: string): string {
  return sessionWindowOpen(iso) ? 'US market open' : 'US market closed'
}

/** formatTimeHHMM(seconds since midnight, UTC) */
export function fmtHmTime(minutes: number): string {
  const h = Math.floor(minutes / 60)
  const m = Math.floor(minutes % 60)
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`
}
