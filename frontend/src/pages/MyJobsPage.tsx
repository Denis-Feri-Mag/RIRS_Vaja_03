import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import { date, jobStatusLabel } from '../lib/format'
import type { JobListItem } from '../lib/types'

export default function MyJobsPage() {
  const { data, isLoading } = useQuery({ queryKey: ['my-jobs'], queryFn: () => api<JobListItem[]>('/job-requests/mine') })

  return (
    <div>
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-xl font-semibold text-sage-600">Moja povpraševanja</h1>
        <Link to="/povprasevanja/novo" className="btn">Novo povpraševanje</Link>
      </div>
      {isLoading && <p>Nalagam...</p>}
      {data?.length === 0 && <p className="card">Še nimate povpraševanj.</p>}
      <div className="space-y-2">
        {data?.map((j) => (
          <Link key={j.id} to={`/povprasevanja/${j.id}`} className="card block hover:bg-slate-50">
            <div className="flex justify-between">
              <span className="font-medium">{j.title}</span>
              <span className="text-sm">{jobStatusLabel[j.status]}</span>
            </div>
            <p className="text-sm text-slate-600">
              {j.category_name}, {j.city}, objavljeno {date(j.created_at)}, ponudb: {j.quote_count}
            </p>
          </Link>
        ))}
      </div>
    </div>
  )
}
