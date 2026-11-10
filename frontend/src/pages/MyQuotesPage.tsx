import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { date, euro, jobStatusLabel, quoteStatusLabel } from '../lib/format'
import type { MyQuoteItem } from '../lib/types'

export default function MyQuotesPage() {
  const { data, isLoading } = useQuery({ queryKey: ['my-quotes'], queryFn: () => api<MyQuoteItem[]>('/quotes/mine') })
  return (
    <div>
      <h1 className="mb-4 text-xl font-semibold text-sage-600">Moje ponudbe</h1>
      {isLoading && <p>Nalagam...</p>}
      {data?.length === 0 && <p className="card">Še niste oddali nobene ponudbe.</p>}
      <div className="space-y-2">
        {data?.map((i) => (
          <Link key={i.quote.id} to={`/povprasevanja/${i.quote.job_request_id}`} className="card block hover:bg-slate-50">
            <div className="flex justify-between">
              <span className="font-medium">{i.job_title}</span>
              <span className="text-sm">Ponudba: {quoteStatusLabel[i.quote.status]}</span>
            </div>
            <p className="text-sm text-slate-600">
              {i.job_city}, {euro(i.quote.price)}, začetek {date(i.quote.earliest_start)}, delo: {jobStatusLabel[i.job_status]}
            </p>
          </Link>
        ))}
      </div>
    </div>
  )
}
