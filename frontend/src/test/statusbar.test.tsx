import { render, screen } from '@testing-library/react'
import { StatusBar } from '../components/Timeline'

describe('trak s statusi', () => {
  it('QUOTED brez aktivnih ponudb pokaže Čaka na ponudbe', () => {
    render(<StatusBar status="QUOTED" activeQuotes={0} />)
    expect(screen.getByText('Čaka na ponudbe')).toHaveClass('bg-sage-600')
    expect(screen.queryByText('Prejete ponudbe')).toBeNull()
  })
  it('QUOTED z ponudbami pokaže Prejete ponudbe', () => {
    render(<StatusBar status="QUOTED" activeQuotes={2} />)
    expect(screen.getByText('Prejete ponudbe')).toHaveClass('bg-sage-600')
  })
})
