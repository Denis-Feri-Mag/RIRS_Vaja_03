import type { ReactNode } from 'react'
import { render } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router-dom'
import { AuthProvider } from '../lib/auth'

export function renderPage(ui: ReactNode) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <AuthProvider>{ui}</AuthProvider>
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

/** Fake fetch that answers by path. */
export function mockFetch(routes: Record<string, unknown>) {
  const calls: { url: string; init?: RequestInit }[] = []
  globalThis.fetch = vi.fn(async (url: string | URL | Request, init?: RequestInit) => {
    const u = String(url)
    calls.push({ url: u, init })
    const key = Object.keys(routes).find((k) => u.includes(k))
    const body = key ? routes[key] : { code: 'NOT_FOUND', message: 'Ne obstaja.' }
    return new Response(JSON.stringify(body), { status: key ? 200 : 404 })
  }) as typeof fetch
  return calls
}
