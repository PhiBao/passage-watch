export default function Header() {
  return (
    <header className="site-header">
      <div className="brand">
        <svg className="brand-mark" viewBox="0 0 32 32" aria-hidden="true">
          <path d="M6 16h8M14 16c4 0 4-8 12-8M14 16c4 0 4 8 12 8" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" />
          <circle cx="6" cy="16" r="2.6" fill="var(--action)" />
          <circle cx="26" cy="8" r="2.6" fill="var(--action)" />
          <circle cx="26" cy="24" r="2.6" fill="var(--action)" />
        </svg>
        <div>
          <div className="brand-name">Passage Watch</div>
          <div className="brand-tag">Choose what your tokenized holding becomes.</div>
        </div>
      </div>
      <nav className="site-nav" aria-label="Primary">
        <a href="#method">Method</a>
        <a href="#sources">Sources</a>
      </nav>
    </header>
  )
}
