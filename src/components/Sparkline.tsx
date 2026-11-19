interface Point {
  price_usd: number
  timestamp: string
}

export default function Sparkline({ points, width = 720, height = 120 }: {
  points: Point[]
  width?: number
  height?: number
}) {
  if (points.length < 2) return null

  const pad = 8
  const prices = points.map((p) => p.price_usd)
  const min = Math.min(...prices)
  const max = Math.max(...prices)
  const span = max - min || 1
  const stepX = (width - pad * 2) / (points.length - 1)

  const coords = points.map((p, i) => {
    const x = pad + i * stepX
    const y = pad + (1 - (p.price_usd - min) / span) * (height - pad * 2)
    return { x, y, ...p }
  })

  const line = coords.map((c) => `${c.x.toFixed(2)},${c.y.toFixed(2)}`).join(' ')
  const area = `${pad},${height - pad} ${line} ${width - pad},${height - pad}`

  const last = coords[coords.length - 1]
  const first = coords[0]
  const up = last.price_usd >= first.price_usd

  return (
    <svg
      className="sparkline"
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      role="img"
      aria-label={`Price trend over ${points.length} observations`}
    >
      <polygon points={area} fill="rgba(60,224,129,0.09)" />
      <polyline
        points={line}
        fill="none"
        stroke={up ? 'var(--action)' : 'var(--caution)'}
        strokeWidth="2"
        strokeLinejoin="round"
        strokeLinecap="round"
        vectorEffect="non-scaling-stroke"
      />
      <circle cx={last.x} cy={last.y} r="3.5" fill={up ? 'var(--action)' : 'var(--caution)'} />
      <title>{`${points[0].timestamp.slice(0, 10)} → ${points[points.length - 1].timestamp.slice(0, 10)} · $${min.toFixed(2)}–$${max.toFixed(2)}`}</title>
    </svg>
  )
}
