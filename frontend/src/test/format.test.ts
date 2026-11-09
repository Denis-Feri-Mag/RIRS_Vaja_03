import { date } from '../lib/format'

describe('datumi', () => {
  it('datum brez ure se ne premakne zaradi časovnega pasu brskalnika', () => {
    const old = process.env.TZ
    process.env.TZ = 'America/Los_Angeles'
    expect(date('2026-11-10')).toBe('10. 11. 2026')
    process.env.TZ = old
  })
})
