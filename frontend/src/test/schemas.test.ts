import { jobSchema } from '../pages/NewJobPage'

const job = {
  category_id: 'c1',
  title: 'Pušča pipa',
  description: 'V kuhinji pušča pipa že nekaj dni.',
  city: 'Maribor',
  postal_code: '2000',
}

describe('US-02 obrazec za povpraševanje', () => {
  it('sprejme pravilne podatke', () => {
    expect(jobSchema.safeParse(job).success).toBe(true)
  })
  it('preveri dolžine in poštno številko', () => {
    expect(jobSchema.safeParse({ ...job, title: 'Pipa' }).success).toBe(false)
    expect(jobSchema.safeParse({ ...job, description: 'kratko' }).success).toBe(false)
    expect(jobSchema.safeParse({ ...job, postal_code: '200' }).success).toBe(false)
  })
  it('budget_min mora biti manjši ali enak budget_max', () => {
    const r = jobSchema.safeParse({ ...job, budget_min: '100', budget_max: '50' })
    expect(r.success).toBe(false)
    expect(r.error?.issues[0].path).toEqual(['budget_max'])
    expect(jobSchema.safeParse({ ...job, budget_min: '50', budget_max: '50' }).success).toBe(true)
    expect(jobSchema.safeParse({ ...job, budget_min: '', budget_max: '' }).success).toBe(true)
  })
})
