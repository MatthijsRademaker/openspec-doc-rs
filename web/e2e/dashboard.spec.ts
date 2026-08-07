import { expect, type Page, test } from '@playwright/test'

const fixtureChange = 'e2e-dashboard-fixture'

interface BrowserHealth {
  consoleErrors: string[]
  pageErrors: string[]
  failedRequests: string[]
}

function observeBrowserHealth(page: Page): BrowserHealth {
  const health: BrowserHealth = {
    consoleErrors: [],
    pageErrors: [],
    failedRequests: [],
  }
  page.on('console', (message) => {
    if (message.type() === 'error') {
      health.consoleErrors.push(message.text())
    }
  })
  page.on('pageerror', (error) => health.pageErrors.push(error.message))
  page.on('requestfailed', (request) => {
    health.failedRequests.push(`${request.url()} — ${request.failure()?.errorText ?? 'unknown'}`)
  })
  return health
}

function expectHealthy(health: BrowserHealth): void {
  expect(health.consoleErrors, 'unexpected browser console errors').toEqual([])
  expect(health.pageErrors, 'unexpected page errors').toEqual([])
  expect(health.failedRequests, 'failed asset or API requests').toEqual([])
}

test('loads deterministic fixture index data and has a healthy browser', async ({ page }) => {
  const health = observeBrowserHealth(page)
  await page.goto('/')

  await expect(page.getByRole('heading', { name: 'openspec-doc' })).toBeVisible()
  await expect(page.getByRole('link', { name: 'E2E Dashboard Fixture' })).toHaveAttribute(
    'href',
    `/changes/${fixtureChange}`,
  )
  await expect(page.getByText(fixtureChange)).toBeVisible()
  await expect(page.getByText('None discovered.')).toBeVisible()
  expectHealthy(health)
})

test('persists the selected theme across reload', async ({ page }) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'Switch to the dark theme' }).click()
  await expect(page.locator('html')).toHaveClass(/dark/)

  await page.reload()

  await expect(page.locator('html')).toHaveClass(/dark/)
  await expect(page.getByRole('button', { name: 'Switch to the light theme' })).toBeVisible()
})

test('keeps the index owned by the Router during history navigation', async ({ page }) => {
  await page.goto('/?router-check=1')

  await expect(page).toHaveURL(/\/?router-check=1$/)
  await expect(page.getByRole('heading', { name: 'openspec-doc' })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'openspec-doc' })).toBeVisible()
})

test('keeps primary index content reachable without horizontal clipping', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('link', { name: 'E2E Dashboard Fixture' })).toBeVisible()

  const layout = await page.evaluate(() => ({
    viewport: window.innerWidth,
    documentWidth: document.documentElement.scrollWidth,
    main: document.querySelector('main')?.getBoundingClientRect().toJSON(),
  }))
  expect(layout.documentWidth, 'page must not overflow horizontally').toBeLessThanOrEqual(
    layout.viewport,
  )
  expect(layout.main).not.toBeNull()
  expect(layout.main?.left).toBeGreaterThanOrEqual(0)
  expect(layout.main?.right).toBeLessThanOrEqual(layout.viewport)
})
