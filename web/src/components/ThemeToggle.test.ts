import { fireEvent, render, screen } from '@testing-library/vue'
import { describe, expect, it } from 'vitest'

import ThemeToggle from '@/components/ThemeToggle.vue'

describe('ThemeToggle', () => {
  it('starts on the light theme when no preference is stored', () => {
    render(ThemeToggle)

    expect(screen.getByRole('button', { name: 'Switch to the dark theme' })).toBeTruthy()
  })

  it('switches to dark and persists the choice', async () => {
    render(ThemeToggle)

    await fireEvent.click(screen.getByRole('button', { name: 'Switch to the dark theme' }))

    expect(document.documentElement.classList.contains('dark')).toBe(true)
    expect(window.localStorage.getItem('openspec-doc-theme')).toBe('dark')
    expect(screen.getByRole('button', { name: 'Switch to the light theme' })).toBeTruthy()
  })

  it('switches back to light', async () => {
    render(ThemeToggle)

    await fireEvent.click(screen.getByRole('button', { name: 'Switch to the dark theme' }))
    await fireEvent.click(screen.getByRole('button', { name: 'Switch to the light theme' }))

    expect(document.documentElement.classList.contains('dark')).toBe(false)
    expect(window.localStorage.getItem('openspec-doc-theme')).toBe('light')
  })
})
