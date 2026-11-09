import { createContext, useContext, useState, type ReactNode } from 'react'
import { session } from './api'
import type { User } from './types'

interface AuthState {
  user: User | null
  login: (token: string, user: User) => void
  logout: () => void
}

const AuthContext = createContext<AuthState | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(() => session.user())
  const value: AuthState = {
    user,
    login(token, u) {
      session.save(token, u)
      setUser(u)
    },
    logout() {
      session.clear()
      setUser(null)
    },
  }
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth outside AuthProvider')
  return ctx
}
