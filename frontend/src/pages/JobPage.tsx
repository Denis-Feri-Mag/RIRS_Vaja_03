import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { api, ApiError } from '../lib/api'
import { useAuth } from '../lib/auth'
import { budget, date, euro, jobStatusText, quoteStatusLabel, term } from '../lib/format'
import type { Job, ProviderProfile, Quote } from '../lib/types'
import QuoteForm from '../components/QuoteForm'
import QuotesTable from '../components/QuotesTable'

export default function JobPage() {
  const { id = '' } = useParams()
  const { user } = useAuth()
  const qc = useQueryClient()
  const navigate = useNavigate()
  const job = useQuery({ queryKey: ['job', id], queryFn: () => api<Job>(`/job-requests/${id}`) })
  const isProvider = user?.role === 'PROVIDER'
  const myQuote = useQuery({
    queryKey: ['my-quote', id],
    queryFn: () => api<Quote | null>(`/job-requests/${id}/my-quote`),
    enabled: isProvider,
  })
  const profile = useQuery({
    queryKey: ['profile'],
    queryFn: () => api<ProviderProfile>('/providers/me'),
    enabled: isProvider,
  })
  const verified = profile.data?.verification_status === 'VERIFIED'
  const [editing, setEditing] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refresh = () => qc.invalidateQueries()

  if (job.isLoading) return <p>Nalagam...</p>
  if (job.error) return <p className="card">{(job.error as Error).message}</p>
  const j = job.data!

  const withdraw = async (q: Quote) => {
    setError(null)
    try {
      await api(`/quotes/${q.id}/withdraw`, { method: 'POST' })
      await refresh()
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'Napaka.')
    }
  }

  const quote = myQuote.data
  const offerMode =
    isProvider && !j.is_accepted_provider && verified && ['OPEN', 'QUOTED'].includes(j.status) && myQuote.isSuccess && (!quote || editing)

  if (offerMode) {
    return (
      <div>
        <p className="mb-2 text-sm text-slate-500">
          <Link to="/dela" className="hover:underline">Ustrezna dela</Link> › {j.title}
        </p>
        <h1 className="mb-6 text-2xl font-semibold text-sage-600">{editing ? 'Urejanje ponudbe' : 'Oddaja ponudbe'}</h1>
        <div className="grid gap-8 md:grid-cols-2">
          <div className="card self-start text-sm">
            <h2 className="mb-2 text-lg font-semibold">{j.title}</h2>
            <span className="mb-3 inline-block rounded-full bg-sage-100 px-3 py-0.5 text-xs text-sage-800">
              {jobStatusText(j.status, j.active_quote_count)}
            </span>
            <div className="space-y-1">
              <p>Kategorija: {j.category_name}</p>
              <p>Kraj: {j.city}, {j.postal_code}</p>
              <p>Želeni termin: {term(j.preferred_from, j.preferred_to)}</p>
              <p>Proračun: {budget(j.budget_min, j.budget_max)}</p>
            </div>
            <hr className="my-3 border-slate-200" />
            <p className="whitespace-pre-line">{j.description}</p>
            <p className="my-4 rounded border border-dashed border-slate-300 bg-slate-50 p-3 text-slate-600">
              Naslov in telefon stranke boste videli, ko bo stranka sprejela vašo ponudbo.
            </p>
            <p className="text-slate-600">Do zdaj oddane ponudbe: {j.active_quote_count}</p>
          </div>
          <QuoteForm
            jobId={id}
            existing={editing ? quote : null}
            onCancel={() => (editing ? setEditing(false) : navigate('/dela'))}
            onSaved={() => {
              setEditing(false)
              refresh()
            }}
          />
        </div>
      </div>
    )
  }

  const compareMode = j.is_owner && ['OPEN', 'QUOTED'].includes(j.status)
  return (
    <div className="space-y-4">
      {compareMode && (
        <QuotesTable
          jobId={id}
          canAccept={j.status === 'QUOTED'}
          onAccepted={refresh}
          header={
            <>
              <p className="mb-2 text-sm text-slate-500">
                <Link to="/povprasevanja" className="hover:underline">Moja povpraševanja</Link> › {j.title}
              </p>
              <h1 className="flex items-center gap-2 text-2xl font-semibold text-sage-600">
                {j.title}
                <span className="rounded-full bg-sage-100 px-3 py-0.5 text-xs font-normal text-sage-800">
                  {jobStatusText(j.status, j.active_quote_count)}
                </span>
              </h1>
              <p className="text-sm text-slate-600">
                Prejete ponudbe: {j.active_quote_count} · poteče {date(j.expires_at)}
              </p>
            </>
          }
        />
      )}
      <div className="card">
        {compareMode ? (
          <h2 className="text-lg font-semibold">Podatki o povpraševanju</h2>
        ) : (
          <h1 className="text-xl font-semibold text-sage-600">{j.title}</h1>
        )}
        <p className="mb-2 text-sm text-slate-600">
          {j.category_name}, {j.postal_code} {j.city}, status: {jobStatusText(j.status, j.active_quote_count)}
        </p>
        <p className="mb-3 whitespace-pre-line">{j.description}</p>
        <div className="grid gap-1 text-sm md:grid-cols-2">
          <p>Termin: {term(j.preferred_from, j.preferred_to)}</p>
          <p>Proračun: {budget(j.budget_min, j.budget_max)}</p>
          {j.address_line && <p>Naslov: {j.address_line}</p>}
          {isProvider && <p>Do zdaj oddane ponudbe: {j.active_quote_count}</p>}
        </div>
      </div>

      {j.provider && (
        <div className="card text-sm">
          <h2 className="mb-1 font-semibold">Izbrani mojster</h2>
          <p>{j.provider.business_name ?? j.provider.full_name}</p>
          <p>{j.provider.full_name}, telefon: {j.provider.phone ?? 'ni vpisan'}</p>
        </div>
      )}
      {j.customer && (
        <div className="card text-sm">
          <h2 className="mb-1 font-semibold">Stranka</h2>
          <p>{j.customer.full_name}, telefon: {j.customer.phone ?? 'ni vpisan'}</p>
        </div>
      )}

      {j.is_owner && !compareMode && <div className="card"><QuotesTable jobId={id} canAccept={false} onAccepted={refresh} /></div>}

      {isProvider && !j.is_accepted_provider && (
        <div className="card">
          <h2 className="mb-3 font-semibold">Vaša ponudba</h2>
          {!quote && profile.data && !verified && (
            <p className="rounded border border-amber-300 bg-amber-50 text-amber-900 p-3 text-sm" role="status">Vaš profil še čaka na preverjanje. Ponudb še ne morete oddati.</p>
          )}
          {quote && (
            <div className="space-y-2 text-sm">
              <p>
                {euro(quote.price)}, {quote.estimated_hours} h, začetek {date(quote.earliest_start)}, status: {quoteStatusLabel[quote.status]}
              </p>
              {quote.status === 'PENDING' && (
                <div className="flex gap-2">
                  <button className="btn-light" onClick={() => setEditing(true)}>Uredi</button>
                  <button className="btn-light" onClick={() => withdraw(quote)}>Umakni</button>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {error && <p className="error" role="alert">{error}</p>}
    </div>
  )
}
