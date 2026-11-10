import { useState, type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { api, ApiError } from '../lib/api'
import { date, euro, quoteStatusLabel } from '../lib/format'
import type { Quote } from '../lib/types'

interface Props {
  jobId: string
  canAccept: boolean
  onAccepted: () => void
  header?: ReactNode
}

export default function QuotesTable({ jobId, canAccept, onAccepted, header }: Props) {
  const [sort, setSort] = useState<'price' | 'start'>('price')
  const [confirm, setConfirm] = useState<Quote | null>(null)
  const [error, setError] = useState<string | null>(null)
  const { data, refetch } = useQuery({ queryKey: ['quotes', jobId, sort], queryFn: () => api<Quote[]>(`/job-requests/${jobId}/quotes?sort=${sort}`) })

  const accept = async (q: Quote) => {
    setError(null)
    try {
      await api(`/quotes/${q.id}/accept`, { method: 'POST' })
      setConfirm(null)
      await refetch()
      onAccepted()
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'Napaka.')
    }
  }

  const sortSelect = (
    <div>
      <label className="label" htmlFor="quote-sort">Razvrsti po</label>
      <select id="quote-sort" className="input" value={sort} onChange={(e) => setSort(e.target.value as 'price' | 'start')}>
        <option value="price">Ceni (naraščajoče)</option>
        <option value="start">Terminu (najprej)</option>
      </select>
    </div>
  )

  return (
    <div>
      <div className="mb-4 flex flex-wrap items-end justify-between gap-4">
        <div>{header ?? <h2 className="font-semibold">Ponudbe</h2>}</div>
        <div className="w-56">{sortSelect}</div>
      </div>
      {data?.length === 0 && <p className="card text-sm text-slate-600">Ponudb še ni.</p>}
      {data && data.length > 0 && (
        <div className="overflow-x-auto rounded-lg border border-slate-200 bg-white">
          <table className="w-full text-left text-sm">
            <thead className="bg-slate-100 text-slate-700">
              <tr>
                <th className="px-3 py-2">Mojster</th>
                <th className="px-3 py-2">Cena</th>
                <th className="px-3 py-2">Trajanje</th>
                <th className="px-3 py-2">Začetek</th>
                <th className="px-3 py-2">Ocena</th>
                <th className="px-3 py-2" />
              </tr>
            </thead>
            <tbody>
              {data.map((q) => (
                <tr key={q.id} className="border-t border-slate-200 align-top">
                  <td className="px-3 py-3">
                    <p className="font-semibold">{q.provider_name}</p>
                    {q.message && <p className="text-xs text-slate-500">Sporočilo: {q.message}</p>}
                  </td>
                  <td className="px-3 py-3">{euro(q.price)}</td>
                  <td className="px-3 py-3">{q.estimated_hours} h</td>
                  <td className="px-3 py-3">{date(q.earliest_start)}</td>
                  <td className="px-3 py-3">★ {q.provider_avg_rating.toFixed(1).replace('.', ',')} ({q.provider_completed_jobs} del)</td>
                  <td className="px-3 py-3 text-right">
                    {canAccept && q.status === 'PENDING' ? (
                      <button className="btn" onClick={() => setConfirm(q)}>Sprejmi</button>
                    ) : (
                      <span className="text-slate-600">{quoteStatusLabel[q.status]}</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {error && <p className="error" role="alert">{error}</p>}
      {confirm && (
        <div className="fixed inset-0 z-10 flex items-center justify-center bg-black/30" role="dialog" aria-labelledby="accept-title">
          <div className="card w-full max-w-md">
            <h2 id="accept-title" className="mb-3 text-lg font-semibold">Sprejem ponudbe</h2>
            <p className="mb-3">Ali res želite sprejeti ponudbo? Ostale ponudbe bodo zavrnjene.</p>
            <p className="mb-5 font-semibold">
              {confirm.provider_name} · {euro(confirm.price)} · {date(confirm.earliest_start)}
            </p>
            <div className="flex justify-end gap-3">
              <button className="btn-light" onClick={() => setConfirm(null)}>Prekliči</button>
              <button className="btn" onClick={() => accept(confirm)}>Da, sprejmi</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
