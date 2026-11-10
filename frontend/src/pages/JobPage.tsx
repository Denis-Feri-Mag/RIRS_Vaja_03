import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { api, ApiError } from '../lib/api'
import { useAuth } from '../lib/auth'
import { budget, date, euro, jobStatusText, quoteStatusLabel, term } from '../lib/format'
import type { HistoryItem, Job, JobStatus, ProviderProfile, Quote } from '../lib/types'
import QuoteForm from '../components/QuoteForm'
import QuotesTable from '../components/QuotesTable'
import Timeline, { StatusBar, Stepper } from '../components/Timeline'

export default function JobPage() {
  const { id = '' } = useParams()
  const { user } = useAuth()
  const qc = useQueryClient()
  const navigate = useNavigate()
  const job = useQuery({ queryKey: ['job', id], queryFn: () => api<Job>(`/job-requests/${id}`) })
  const history = useQuery({ queryKey: ['history', id], queryFn: () => api<HistoryItem[]>(`/job-requests/${id}/history`) })
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
  const acceptedQuotes = useQuery({
    queryKey: ['quotes', id, 'price'],
    queryFn: () => api<Quote[]>(`/job-requests/${id}/quotes?sort=price`),
    enabled: user?.role === 'CUSTOMER',
  })
  const verified = profile.data?.verification_status === 'VERIFIED'
  const [editing, setEditing] = useState(false)
  const [reason, setReason] = useState('')
  const [error, setError] = useState<string | null>(null)

  const refresh = () => qc.invalidateQueries()

  if (job.isLoading) return <p>Nalagam...</p>
  if (job.error) return <p className="card">{(job.error as Error).message}</p>
  const j = job.data!

  const changeStatus = async (status: JobStatus, note?: string) => {
    setError(null)
    try {
      await api(`/job-requests/${id}/status`, { method: 'POST', body: { status, note } })
      setReason('')
      await refresh()
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'Napaka.')
    }
  }

  const withdraw = async (q: Quote) => {
    setError(null)
    try {
      await api(`/quotes/${q.id}/withdraw`, { method: 'POST' })
      await refresh()
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'Napaka.')
    }
  }

  const canCancel = j.is_owner && ['OPEN', 'QUOTED', 'ACCEPTED'].includes(j.status)
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
  const trackMode = (j.is_owner ? !!j.provider : j.is_accepted_provider) && !['OPEN', 'QUOTED'].includes(j.status)
  const accepted = j.is_owner ? acceptedQuotes.data?.find((q) => q.status === 'ACCEPTED') : quote

  if (trackMode) {
    const other = j.is_owner ? j.provider : j.customer
    return (
      <div className="space-y-8">
        <div>
          <p className="mb-2 text-sm text-slate-500">
            <Link to={j.is_owner ? '/povprasevanja' : '/ponudbe'} className="hover:underline">
              {j.is_owner ? 'Moja povpraševanja' : 'Moje ponudbe'}
            </Link>{' '}
            › {j.title}
          </p>
          <h1 className="flex items-center gap-2 text-2xl font-semibold text-sage-600">
            {j.title}
            <span className="rounded-full bg-sage-100 px-3 py-0.5 text-xs font-normal text-sage-800">{jobStatusText(j.status)}</span>
          </h1>
        </div>
        {['CANCELLED', 'EXPIRED'].includes(j.status) ? null : <Stepper status={j.status} />}
        <div className="grid gap-6 md:grid-cols-2">
          <div className="card self-start text-sm">
            <h2 className="mb-3 text-lg font-semibold">{j.is_owner ? 'Izbrani mojster' : 'Stranka'}</h2>
            <div className="mb-4 flex items-center gap-3">
              <span className="h-10 w-10 rounded-full bg-sage-100" />
              <div>
                <p className="font-semibold">{other?.business_name ?? other?.full_name}</p>
                {j.is_owner && accepted && (
                  <p className="text-slate-500">
                    ★ {accepted.provider_avg_rating.toFixed(1).replace('.', ',')} · {accepted.provider_completed_jobs} opravljenih del
                  </p>
                )}
              </div>
            </div>
            <div className="space-y-1">
              {accepted && <p>Cena: {euro(accepted.price)}</p>}
              {accepted && <p>Dogovorjen začetek: {date(accepted.earliest_start)}</p>}
              <p>Telefon: {other?.phone ?? 'ni vpisan'}</p>
              {j.address_line && <p>Naslov: {j.address_line}, {j.postal_code} {j.city}</p>}
            </div>
            {j.is_owner && ['ACCEPTED', 'IN_PROGRESS'].includes(j.status) && (
              <p className="mt-4 rounded border border-dashed border-slate-300 bg-slate-50 p-3 text-slate-600">
                Ko mojster označi delo kot zaključeno, bo delo zaključeno tudi na vaši strani.
              </p>
            )}
          </div>
          <div className="card self-start">
            <h2 className="mb-4 text-lg font-semibold">Zgodovina statusov</h2>
            {history.data && <Timeline items={history.data} />}
          </div>
        </div>

        {error && <p className="error" role="alert">{error}</p>}

        {j.is_accepted_provider && (j.status === 'ACCEPTED' || j.status === 'IN_PROGRESS') && (
          <div className="flex justify-end">
            {j.status === 'ACCEPTED' && <button className="btn" onClick={() => changeStatus('IN_PROGRESS')}>Začni delo</button>}
            {j.status === 'IN_PROGRESS' && <button className="btn" onClick={() => changeStatus('COMPLETED')}>Zaključi delo</button>}
          </div>
        )}

        {j.is_owner && ['ACCEPTED', 'IN_PROGRESS'].includes(j.status) &&
          (canCancel ? (
            <div className="space-y-2">
              <label className="label" htmlFor="reason">Razlog za preklic, vsaj 10 znakov</label>
              <textarea id="reason" className="input" rows={2} value={reason} onChange={(e) => setReason(e.target.value)} />
              <button className="btn-light" disabled={reason.trim().length < 10} onClick={() => changeStatus('CANCELLED', reason)}>
                Prekliči delo
              </button>
            </div>
          ) : (
            <div className="flex items-center gap-4">
              <button className="btn-light" disabled>Prekliči delo</button>
              <p className="text-sm text-slate-600">Preklic ni več mogoč, ker je delo že v teku.</p>
            </div>
          ))}
      </div>
    )
  }

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
        {!compareMode && <StatusBar status={j.status} activeQuotes={j.active_quote_count} />}
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

      {j.is_accepted_provider && (j.status === 'ACCEPTED' || j.status === 'IN_PROGRESS') && (
        <div className="card flex gap-2">
          {j.status === 'ACCEPTED' && <button className="btn" onClick={() => changeStatus('IN_PROGRESS')}>Začni delo</button>}
          {j.status === 'IN_PROGRESS' && <button className="btn" onClick={() => changeStatus('COMPLETED')}>Zaključi delo</button>}
        </div>
      )}

      {j.is_owner && !['CANCELLED', 'EXPIRED', 'COMPLETED', 'REVIEWED'].includes(j.status) && (
        <div className="card">
          <h2 className="mb-2 font-semibold">Preklic</h2>
          {canCancel ? (
            <div className="space-y-2">
              <label className="label" htmlFor="reason">Razlog za preklic, vsaj 10 znakov</label>
              <textarea id="reason" className="input" rows={2} value={reason} onChange={(e) => setReason(e.target.value)} />
              <button className="btn-light" disabled={reason.trim().length < 10} onClick={() => changeStatus('CANCELLED', reason)}>
                Prekliči delo
              </button>
            </div>
          ) : (
            <p className="text-sm text-slate-600">Preklic ni več mogoč, ker je delo že v teku.</p>
          )}
        </div>
      )}

      {error && <p className="error" role="alert">{error}</p>}

      <div className="card">
        <h2 className="mb-3 font-semibold">Zgodovina statusov</h2>
        {history.data && <Timeline items={history.data} />}
      </div>
    </div>
  )
}
