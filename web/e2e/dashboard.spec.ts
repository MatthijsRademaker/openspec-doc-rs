import { expect, type Page, test } from '@playwright/test'

const fixtureChange = 'implement-observatory-design-system-with-a-realistically-long-identifier'
const fixtureTitle = 'Observatory Design System Fixture'
const fixtureSession = '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74'
const fixtureSessionTitle = 'Agent-guided observatory exploration session'

function parseUrl(raw: string, context: string): URL {
  try {
    return new URL(raw)
  } catch (error) {
    throw new Error(`Invalid ${context} URL: ${raw}`, { cause: error })
  }
}

const embeddedOrigin = parseUrl(
  process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8792',
  'embedded origin',
).origin

interface BrowserHealth {
  consoleErrors: string[]
  pageErrors: string[]
  failedRequests: string[]
  externalRequests: string[]
}

function observeBrowserHealth(page: Page): BrowserHealth {
  const health: BrowserHealth = {
    consoleErrors: [],
    pageErrors: [],
    failedRequests: [],
    externalRequests: [],
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
  page.on('request', (request) => {
    const url = parseUrl(request.url(), 'browser request')
    if ((url.protocol === 'http:' || url.protocol === 'https:') && url.origin !== embeddedOrigin) {
      health.externalRequests.push(request.url())
    }
  })
  return health
}

function expectHealthy(health: BrowserHealth): void {
  expect(health.consoleErrors, 'unexpected browser console errors').toEqual([])
  expect(health.pageErrors, 'unexpected page errors').toEqual([])
  expect(health.failedRequests, 'failed asset or API requests').toEqual([])
  expect(health.externalRequests, 'dashboard must stay offline and same-origin').toEqual([])
}

test('renders realistic scope data through complete instrument registers', async ({ page }) => {
  const health = observeBrowserHealth(page)
  await page.goto('/')

  await expect(page.getByRole('heading', { name: 'openspec-doc' })).toBeVisible()
  await expect(page.getByRole('link', { name: fixtureTitle })).toHaveAttribute(
    'href',
    `/changes/${fixtureChange}`,
  )
  await expect(page.getByText(fixtureChange)).toBeVisible()
  await expect(page.getByText('1 open')).toBeVisible()
  await expect(page.getByText('comment-resolution')).toBeVisible()
  await expect(page.getByRole('link', { name: fixtureSessionTitle })).toHaveAttribute(
    'href',
    `/sessions/${fixtureSession}`,
  )
  await expect(page.getByText('most recently active')).toBeVisible()
  expectHealthy(health)
})

test('ignores stale theme storage and loads all three font roles offline', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('openspec-doc-theme', 'light'))
  const health = observeBrowserHealth(page)
  await page.goto('/')
  await page.evaluate(() => document.fonts.ready)

  await expect(page.getByRole('button', { name: /theme/i })).toHaveCount(0)
  const identity = await page.evaluate(() => ({
    background: getComputedStyle(document.body).backgroundColor,
    displayLoaded: document.fonts.check('16px "Cormorant Garamond"'),
    proseLoaded: document.fonts.check('16px "IBM Plex Sans Variable"'),
    monoLoaded: document.fonts.check('16px "IBM Plex Mono"'),
  }))
  expect(identity).toEqual({
    background: 'rgb(11, 12, 14)',
    displayLoaded: true,
    proseLoaded: true,
    monoLoaded: true,
  })
  expectHealthy(health)
})

test('keeps index owned by Router during history navigation', async ({ page }) => {
  await page.goto('/?router-check=1')

  await expect(page).toHaveURL(/\/?router-check=1$/)
  await expect(page.getByRole('heading', { name: 'openspec-doc' })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'openspec-doc' })).toBeVisible()
})

test('keeps identifiers, metadata, states, and links in narrow flow', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('link', { name: fixtureTitle })).toBeVisible()

  for (const value of [
    fixtureChange,
    '1 open',
    'comment-resolution',
    'most recently active',
    'Modified',
    'Open comments',
    'Verdict',
  ]) {
    await expect(page.getByText(value).first()).toBeVisible()
  }

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

test('shows keyboard focus and makes reduced-motion state changes immediate', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' })
  await page.goto('/')

  await page.keyboard.press('Tab')
  const link = page.getByRole('link', { name: fixtureSessionTitle })
  await expect(link).toBeFocused()
  const focus = await link.evaluate((element) => {
    const style = getComputedStyle(element)
    return { style: style.outlineStyle, width: style.outlineWidth }
  })
  expect(focus.style).toBe('solid')
  expect(focus.width).toBe('2px')

  const motion = await page
    .locator('.scope-entry')
    .first()
    .evaluate((element) => {
      const style = getComputedStyle(element)
      return { transform: style.transform, transitionDuration: style.transitionDuration }
    })
  expect(motion.transform).toBe('none')
  expect(motion.transitionDuration).toBe('0s')
})

test('captures embedded observatory index for source-board review', async ({ page }, testInfo) => {
  const health = observeBrowserHealth(page)
  await page.goto('/')
  await expect(page.getByRole('link', { name: fixtureTitle })).toBeVisible()

  await page.screenshot({ path: testInfo.outputPath('observatory-index.png'), fullPage: true })
  expectHealthy(health)
})
