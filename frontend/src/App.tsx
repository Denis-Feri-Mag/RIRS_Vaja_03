import { Link, Navigate, Route, Routes } from 'react-router-dom'

export default function App() {
  return (
    <div className="min-h-screen">
      <header className="border-b-4 border-sage-600 bg-white">
        <nav className="mx-auto flex max-w-5xl flex-wrap items-end gap-1 px-4 pt-3 text-sm">
          <Link to="/" className="mr-6 mb-2 text-lg font-semibold text-sage-600">
            Popravi.si
          </Link>
        </nav>
      </header>
      <main className="mx-auto max-w-5xl px-4 py-6">
        <Routes>
          <Route path="/" element={<p className="card">Popravi.si je v izdelavi.</p>} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
    </div>
  )
}
