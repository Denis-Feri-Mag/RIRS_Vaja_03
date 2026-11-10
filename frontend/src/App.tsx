import { useEffect, type ReactNode } from 'react'
import { Link, NavLink, Navigate, Route, Routes, useNavigate } from 'react-router-dom'
import { setUnauthorizedHandler } from './lib/api'
import { useAuth } from './lib/auth'
import type { Role } from './lib/types'
import LoginPage from './pages/LoginPage'
import RegisterPage from './pages/RegisterPage'
import MyJobsPage from './pages/MyJobsPage'
import NewJobPage from './pages/NewJobPage'
import JobPage from './pages/JobPage'
import MatchingPage from './pages/MatchingPage'
import ProfilePage from './pages/ProfilePage'

function RequireRole({ roles, children }: { roles: Role[]; children: ReactNode }) {
  const { user } = useAuth()
  if (!user) return <Navigate to="/prijava" replace />
  if (!roles.includes(user.role)) return <Navigate to="/" replace />
  return <>{children}</>
}

function Home() {
  const { user } = useAuth()
  if (!user) return <Navigate to="/prijava" replace />
  if (user.role === 'PROVIDER') return <Navigate to="/dela" replace />
  if (user.role === 'CUSTOMER') return <Navigate to="/povprasevanja" replace />
  return <p className="card">Administracija pride v kasnejši različici.</p>
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
          {user?.role === 'CUSTOMER' && (
            <>
              <NavLink to="/povprasevanja" end className="nav-tab">Moja povpraševanja</NavLink>
              <NavLink to="/povprasevanja/novo" end className="nav-tab">Novo povpraševanje</NavLink>
            </>
          )}
          {user?.role === 'PROVIDER' && (
            <>
              <NavLink to="/dela" end className="nav-tab">Ustrezna dela</NavLink>
              <NavLink to="/profil" end className="nav-tab">Profil</NavLink>
            </>
          )}
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
          <Route path="/povprasevanja" element={<RequireRole roles={['CUSTOMER']}><MyJobsPage /></RequireRole>} />
          <Route path="/povprasevanja/novo" element={<RequireRole roles={['CUSTOMER']}><NewJobPage /></RequireRole>} />
          <Route path="/povprasevanja/:id" element={<RequireRole roles={['CUSTOMER', 'PROVIDER', 'ADMIN']}><JobPage /></RequireRole>} />
          <Route path="/dela" element={<RequireRole roles={['PROVIDER']}><MatchingPage /></RequireRole>} />
          <Route path="/profil" element={<RequireRole roles={['PROVIDER']}><ProfilePage /></RequireRole>} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
    </div>
  )
}
