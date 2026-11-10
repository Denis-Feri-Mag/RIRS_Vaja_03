import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import RegisterPage from '../pages/RegisterPage'
import MatchingPage from '../pages/MatchingPage'
import QuotesTable from '../components/QuotesTable'
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

describe('US-05 tabela ponudb', () => {
  it('pred sprejemom vpraša za potrditev', async () => {
    const calls = mockFetch({
      '/accept': {},
      '/quotes?sort': [
        { id: 'q1', job_request_id: 'j1', provider_id: 'p1', provider_name: 'Marko s.p.', provider_avg_rating: 4.5, provider_completed_jobs: 3,
          price: 60, estimated_hours: 1, earliest_start: '2026-11-10', message: null, status: 'PENDING', created_at: '2026-11-07T10:00:00Z' },
      ],
    })
    const onAccepted = vi.fn()
    renderPage(<QuotesTable jobId="j1" canAccept onAccepted={onAccepted} />)
    const u = userEvent.setup()
    await u.click(await screen.findByRole('button', { name: 'Sprejmi' }))
    expect(screen.getByRole('dialog')).toHaveTextContent('Ali res želite sprejeti ponudbo? Ostale ponudbe bodo zavrnjene.')
    await u.click(screen.getByRole('button', { name: 'Da, sprejmi' }))
    await waitFor(() => expect(onAccepted).toHaveBeenCalled())
    expect(calls.some((c) => c.url.endsWith('/quotes/q1/accept') && c.init?.method === 'POST')).toBe(true)
  })

  it('razvrsti po terminu', async () => {
    const base = { job_request_id: 'j1', provider_avg_rating: 0, provider_completed_jobs: 0, estimated_hours: 1, message: null, status: 'PENDING', created_at: '2026-11-07T10:00:00Z' }
    const a = { ...base, id: 'qa', provider_id: 'pa', provider_name: 'Poceni s.p.', price: 50, earliest_start: '2026-11-20' }
    const b = { ...base, id: 'qb', provider_id: 'pb', provider_name: 'Hitri s.p.', price: 90, earliest_start: '2026-11-10' }
    const calls = mockFetch({ 'sort=start': [b, a], 'sort=price': [a, b] })
    renderPage(<QuotesTable jobId="j1" canAccept={false} onAccepted={() => {}} />)
    const rows = async () => (await screen.findAllByRole('row')).slice(1).map((r) => r.textContent)
    await screen.findByText('Poceni s.p.')
    expect((await rows())[0]).toContain('Poceni s.p.')
    await userEvent.setup().selectOptions(screen.getByRole('combobox'), 'start')
    await waitFor(async () => expect((await rows())[0]).toContain('Hitri s.p.'))
    expect((await rows())[0]).toContain('10. 11. 2026')
    expect(calls.some((c) => c.url.includes('sort=start'))).toBe(true)
  })

  it('brez pravice ni gumba Sprejmi', async () => {
    mockFetch({ '/quotes?sort': [] })
    renderPage(<QuotesTable jobId="j1" canAccept={false} onAccepted={() => {}} />)
    expect(await screen.findByText('Ponudb še ni.')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Sprejmi' })).toBeNull()
  })
})
