import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import RegisterPage from '../pages/RegisterPage'
import MatchingPage from '../pages/MatchingPage'
import { mockFetch, renderPage } from './utils'

afterEach(() => localStorage.clear())

describe('US-01 registracija', () => {
  it('ponudi samo vlogi stranka in izvajalec', () => {
    renderPage(<RegisterPage />)
    expect(screen.getByLabelText('Stranka')).toBeInTheDocument()
    expect(screen.getByLabelText('Izvajalec')).toBeInTheDocument()
    expect(screen.queryByLabelText(/Administrator/)).toBeNull()
  })

  it('pokaže napako za podvojen e-naslov', async () => {
    globalThis.fetch = vi.fn(async () =>
      new Response(JSON.stringify({ code: 'EMAIL_TAKEN', message: 'Račun s tem e-naslovom že obstaja.' }), { status: 409 }),
    ) as typeof fetch
    renderPage(<RegisterPage />)
    const u = userEvent.setup()
    await u.type(screen.getByLabelText(/^E-naslov/), 'ana@primer.si')
    await u.type(screen.getByLabelText(/^Ime in priimek/), 'Ana Novak')
    await u.type(screen.getByLabelText(/^Geslo/), 'geslo1234')
    await u.click(screen.getByLabelText('Stranka'))
    await u.click(screen.getByRole('button', { name: 'Ustvari račun' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('Račun s tem e-naslovom že obstaja.')
  })
})

describe('US-03 seznam ustreznih del', () => {
  const item = {
    id: 'j1', title: 'Pušča pipa', category_name: 'Vodovodna dela', city: 'Maribor', postal_code: '2000',
    preferred_from: null, preferred_to: null, budget_min: null, budget_max: 80, status: 'OPEN', quote_count: 2, created_at: '2026-11-07T10:00:00Z',
  }

  it('označi oddano ponudbo in pokaže obvestilo za nepreverjen profil', async () => {
    mockFetch({
      '/job-requests/matching': {
        items: [{ ...item, quote_sent: true }, { ...item, id: 'j2', title: 'Nova vtičnica', quote_sent: false }],
        page: 1, page_size: 20, total: 2, verification_status: 'PENDING', category_names: ['Vodovodna dela'], service_postal_codes: ['2000'],
      },
    })
    renderPage(<MatchingPage />)
    expect(await screen.findByText('Ponudba oddana')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Oddaj ponudbo' })).toBeDisabled()
    expect(screen.queryByRole('link', { name: 'Oddaj ponudbo' })).toBeNull()
    expect(screen.getByRole('status')).toHaveTextContent('Vaš profil še čaka na preverjanje. Ponudb še ne morete oddati.')
    expect(screen.getAllByText(/ponudb: 2/)).toHaveLength(2)
  })

  it('ima gumb Osveži, ki ponovno naloži seznam', async () => {
    const calls = mockFetch({
      '/job-requests/matching': {
        items: [{ ...item, quote_sent: false }],
        page: 1, page_size: 20, total: 1, verification_status: 'VERIFIED', category_names: ['Vodovodna dela'], service_postal_codes: ['2000'],
      },
    })
    renderPage(<MatchingPage />)
    expect(await screen.findByText(/Maribor 2000/)).toBeInTheDocument()
    const before = calls.length
    await userEvent.setup().click(screen.getByRole('button', { name: 'Osveži' }))
    await waitFor(() => expect(calls.length).toBeGreaterThan(before))
  })
})
