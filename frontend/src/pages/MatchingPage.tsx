import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { budget, dateTime, term } from '../lib/format'
import type { MatchingPage as Page } from '../lib/types'

export default function MatchingPage() {
  const [page, setPage] = useState(1)
  const { data, isLoading, refetch, isFetching } = useQuery({ queryKey: ['matching', page], queryFn: () => api<Page>(`/job-requests/matching?page=${page}`) })
  const verified = data?.verification_status === 'VERIFIED'
  const pages = data ? Math.max(1, Math.ceil(data.total / data.page_size)) : 1

  return (
    <div>
      <div className="mb-2 flex items-center justify-between">
        <h1 className="text-xl font-semibold text-sage-600">Ustrezna dela</h1>
        <div className="flex items-center gap-3 text-sm">
          <span className="text-slate-500">Razvrščeno od najnovejšega</span>
          <button className="btn-light" onClick={() => refetch()} disabled={isFetching}>Osveži</button>
        </div>
      </div>
      {data && (
        <p className="mb-4 text-sm text-slate-600">
          Prikazana so odprta povpraševanja iz vaših kategorij in poštnih številk. Kategorije: {data.category_names.join(', ') || 'ni izbranih'}. Poštne številke: {data.service_postal_codes.join(', ') || 'ni izbranih'}.{' '}
          <Link to="/profil" className="underline">Uredi profil</Link>
        </p>
      )}
      {data && data.verification_status !== 'VERIFIED' && (
        <p className="mb-4 rounded border border-amber-300 bg-amber-50 text-amber-900 p-3 text-sm" role="status">
          Vaš profil še čaka na preverjanje. Ponudb še ne morete oddati.
        </p>
      )}
      {isLoading && <p>Nalagam...</p>}
      {data?.items.length === 0 && <p className="card">Trenutno ni ustreznih povpraševanj.</p>}
      <div className="space-y-2">
        {data?.items.map((j) => (
          <div key={j.id} className="card flex flex-wrap items-center justify-between gap-2">
            <div>
              <p className="font-medium">
                {j.title} <span className="text-xs font-normal text-slate-500">{dateTime(j.created_at)}</span>
              </p>
              <p className="text-sm text-slate-600">
                {j.category_name}, {j.city} {j.postal_code}, termin {term(j.preferred_from, j.preferred_to)}, proračun {budget(j.budget_min, j.budget_max)}, ponudb: {j.quote_count}
              </p>
            </div>
            {j.quote_sent ? (
              <Link to={`/povprasevanja/${j.id}`} className="rounded-full bg-sage-100 px-3 py-1 text-sm text-sage-800">Ponudba oddana</Link>
            ) : verified ? (
              <Link to={`/povprasevanja/${j.id}`} className="btn">Oddaj ponudbo</Link>
            ) : (
              <button className="btn" disabled title="Profil še ni preverjen">Oddaj ponudbo</button>
            )}
          </div>
        ))}
      </div>
      {data && data.total > data.page_size && (
        <div className="mt-4 flex items-center gap-3 text-sm">
          <button className="btn-light" disabled={page <= 1} onClick={() => setPage(page - 1)}>Nazaj</button>
          <span>Stran {page} od {pages}</span>
          <button className="btn-light" disabled={page >= pages} onClick={() => setPage(page + 1)}>Naprej</button>
        </div>
      )}
    </div>
  )
}
