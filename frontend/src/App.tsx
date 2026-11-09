import { useEffect } from 'react'
import { Link, NavLink, Navigate, Route, Routes, useNavigate } from 'react-router-dom'
import { setUnauthorizedHandler } from './lib/api'
import { useAuth } from './lib/auth'
import LoginPage from './pages/LoginPage'
import RegisterPage from './pages/RegisterPage'

function Home() {
  const { user } = useAuth()
  if (!user) return <Navigate to="/prijava" replace />
  return <p className="card">Popravi.si je v izdelavi.</p>
}

export default function App() {
  const { user, logout } = useAuth()
  const navigate = useNavigate()

  useEffect(() => {
    setUnauthorizedHandler(() => {
      logout()
      navigate('/prijava?potekla=1')
    })
  }, [logout, navigate])

  return (
    <div className="min-h-screen">
      <header className="border-b-4 border-sage-600 bg-white">
        <nav className="mx-auto flex max-w-5xl flex-wrap items-end gap-1 px-4 pt-3 text-sm">
          <Link to="/" className="mr-6 mb-2 text-lg font-semibold text-sage-600">
            Popravi.si
          </Link>
          <span className="ml-auto" />
          {user ? (
            <>
              <span className="mr-2 mb-3 text-slate-500">{user.full_name}</span>
              <button
                className="btn-light mb-2 py-1.5"
                onClick={() => {
                  logout()
                  navigate('/prijava')
                }}
              >
                Odjava
              </button>
            </>
          ) : (
            <>
              <NavLink to="/prijava" end className="nav-tab">Prijava</NavLink>
              <NavLink to="/registracija" end className="nav-tab">Registracija</NavLink>
            </>
          )}
        </nav>
      </header>
      <main className="mx-auto max-w-5xl px-4 py-6">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/prijava" element={<LoginPage />} />
          <Route path="/registracija" element={<RegisterPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
    </div>
  )
}
