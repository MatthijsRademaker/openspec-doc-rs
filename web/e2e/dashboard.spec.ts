import { expect, type Page, test } from '@playwright/test'

const fixtureChange = 'implement-observatory-design-system-with-a-realistically-long-identifier'
const fixtureTitle = 'Observatory Design System Fixture'
const fixtureSession = '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74'
const fixtureSessionTitle = 'Agent-guided observatory exploration session'
const untitledFixtureSession = '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d77'
const fixtureScopeCount = { changes: 3, sessions: 9 }

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

  await expect(page.getByRole('heading', { name: 'Changes', level: 1 })).toBeVisible()
  await expect(page.getByText('openspec-doc', { exact: false }).first()).toBeVisible()
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

test('keeps index and scope navigation owned by Router', async ({ page }) => {
  await page.goto('/?router-check=1')

  await expect(page).toHaveURL(/\/?router-check=1$/)
  await expect(page.getByRole('heading', { name: 'Changes', level: 1 })).toBeVisible()
  await page.getByRole('link', { name: fixtureTitle }).click()
  await expect(page).toHaveURL(`/changes/${fixtureChange}`)
  await expect(page.locator('.scope-header h1')).toHaveText(fixtureTitle)
  await page.reload()
  await expect(page.locator('.scope-header h1')).toHaveText(fixtureTitle)
})

test('keeps identifiers, metadata, states, and links in narrow flow', async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'narrow', '390px contract')
  await page.goto('/')
  await expect(page.getByRole('link', { name: fixtureTitle })).toBeVisible()

  for (const value of [
    fixtureChange,
    fixtureSession,
    untitledFixtureSession,
    '1 open',
    'comment-resolution',
    'most recently active',
    'Modified',
    'Open comments',
    'Verdict',
  ]) {
    await expect(page.getByText(value).first()).toBeVisible()
  }

  await expect(page.locator('.index-workbench a')).toHaveCount(
    fixtureScopeCount.changes + fixtureScopeCount.sessions,
  )
  const layout = await page.evaluate(() => {
    const changes = document.querySelector('#changes-title')?.getBoundingClientRect()
    const sessions = document.querySelector('#sessions-register-title')?.getBoundingClientRect()
    const sessionList = document.querySelector('.scope-register--secondary .scope-register__list')
    const sessionStyle = sessionList ? getComputedStyle(sessionList) : null
    const plates = document.querySelector('.index-plates')
    const art = document.querySelector('.index-observation__art')?.getBoundingClientRect()
    return {
      viewport: window.innerWidth,
      documentWidth: document.documentElement.scrollWidth,
      main: document.querySelector('main')?.getBoundingClientRect().toJSON(),
      changesTop: changes?.top,
      sessionsTop: sessions?.top,
      sessionMaxHeight: sessionStyle?.maxHeight,
      sessionOverflow: sessionStyle?.overflowY,
      platesDisplay: plates ? getComputedStyle(plates).display : null,
      artHeight: art?.height,
    }
  })
  expect(layout.documentWidth, 'page must not overflow horizontally').toBeLessThanOrEqual(
    layout.viewport,
  )
  expect(layout.main).not.toBeNull()
  expect(layout.main?.left).toBeGreaterThanOrEqual(0)
  expect(layout.main?.right).toBeLessThanOrEqual(layout.viewport)
  expect(layout.changesTop).toBeLessThan(layout.sessionsTop ?? 0)
  expect(layout.sessionMaxHeight).toBe('none')
  expect(layout.sessionOverflow).toBe('visible')
  expect(layout.platesDisplay).toBe('none')
  expect(layout.artHeight).toBeLessThanOrEqual(176)
})

test('shows keyboard focus and makes reduced-motion state changes immediate', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' })
  await page.goto('/')

  await page.keyboard.press('Tab')
  const link = page.locator('.scope-register--primary a').first()
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

test('keeps Changes initial and every session keyboard-reachable through desktop rail', async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'desktop session rail contract')
  await page.goto('/')

  const changesHeading = page.getByRole('heading', { name: 'Changes', level: 1 })
  const sessionsHeading = page.getByRole('heading', { name: 'Sessions', level: 2 })
  await expect(changesHeading).toBeInViewport()
  await expect(sessionsHeading).toBeVisible()
  await expect(page.locator('.scope-register--primary a').first()).toBeInViewport()
  const changesBeforeSessions = await changesHeading.evaluate((node) => {
    const sessions = document.querySelector('#sessions-register-title')
    return Boolean(
      sessions && node.compareDocumentPosition(sessions) & Node.DOCUMENT_POSITION_FOLLOWING,
    )
  })
  expect(changesBeforeSessions).toBe(true)

  const sessionList = page.locator('.scope-register--secondary .scope-register__list')
  const rail = await sessionList.evaluate((element) => {
    const style = getComputedStyle(element)
    return {
      overflowY: style.overflowY,
      scrollHeight: element.scrollHeight,
      clientHeight: element.clientHeight,
    }
  })
  expect(rail.overflowY).toBe('auto')
  expect(rail.scrollHeight).toBeGreaterThan(rail.clientHeight)

  const links = page.locator('.index-workbench a')
  const expectedHrefs = await links.evaluateAll((elements) =>
    elements.map((element) => (element as HTMLAnchorElement).getAttribute('href')),
  )
  for (let index = 0; index < expectedHrefs.length; index += 1) {
    await page.keyboard.press('Tab')
    const focusedHref = await page.evaluate(() =>
      (document.activeElement as HTMLAnchorElement | null)?.getAttribute('href'),
    )
    expect(focusedHref).toBe(expectedHrefs[index])
    if (focusedHref?.startsWith('/sessions/')) {
      await expect(links.nth(index)).toBeInViewport()
    }
  }
})

test('renders anchored, orphaned, and addressed review state on scope page', async ({ page }) => {
  const health = observeBrowserHealth(page)
  await page.goto(`/changes/${fixtureChange}`)

  await expect(page.getByText('Browser lane must see this proposal.')).toBeVisible()
  const utility = page.locator('.scope-utility')
  await expect(utility.getByText('1 open', { exact: true })).toBeVisible()
  await expect(utility.getByText('1 addressed', { exact: true })).toBeVisible()
  await expect(utility.getByText('1 resolved', { exact: true })).toBeVisible()
  await expect(page.getByText('Not yet delivered')).toBeVisible()

  const repeatedBlocks = page
    .locator('.review-block')
    .filter({ hasText: 'Repeated review target.' })
  await repeatedBlocks.nth(1).getByRole('button', { name: '1 open comment' }).click()
  await expect(page.getByText('Keep repeated occurrence mapping exact.')).toBeVisible()
  await expect(page.getByText('Second occurrence is now explicit.')).toBeVisible()
  await expect(page.getByText('Reviewer', { exact: true }).first()).toBeVisible()
  await expect(page.getByText('Agent', { exact: true }).first()).toBeVisible()
  await expect(page.getByRole('button', { name: 'Reply' })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Resolve' })).toBeVisible()
  await expect(page.getByRole('button', { name: /addressed/i })).toHaveCount(0)

  await page.getByRole('button', { name: /Comment \/ 2 without block/ }).click()
  await expect(page.getByText('Lost anchors must stay reachable.')).toBeVisible()
  await expect(page.getByText('Anchor lost')).toBeVisible()
  await expect(page.getByText('Original text rewritten away.')).toBeVisible()
  await expect(page.getByText(/Agent claims work complete/)).toBeVisible()
  await expect(page.getByRole('button', { name: 'Accept as resolved' })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Reopen' })).toHaveCount(2)
  expectHealthy(health)
})

test('anchors a new comment to second repeated block', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'narrow', 'mutation runs after shared-state assertions')
  await page.goto(`/changes/${fixtureChange}`)
  const repeatedBlocks = page
    .locator('.review-block')
    .filter({ hasText: 'Repeated review target.' })
  const secondBlock = repeatedBlocks.nth(1)
  const blockId = await secondBlock.getAttribute('data-block-id')

  await secondBlock.getByRole('button', { name: /Comment on .* block/ }).click()
  await secondBlock.locator('..').getByLabel('Comment on block').fill('E2E repeated occurrence.')
  await secondBlock.locator('..').getByRole('button', { name: 'Record comment' }).click()

  const created = await page.evaluate(
    async ({ change }) => {
      const response = await fetch(`/api/changes/${change}`)
      if (!response.ok) throw new Error(`scope detail ${response.status}`)
      const detail = (await response.json()) as {
        comments: Array<{
          comment: { body: string; anchor?: { startOffset: number } }
          anchorState: string
          blockId: string | null
        }>
      }
      return detail.comments.find((thread) => thread.comment.body === 'E2E repeated occurrence.')
    },
    { change: fixtureChange },
  )

  expect(created?.anchorState).toBe('exact')
  expect(created?.blockId).toBe(blockId)
  expect(created?.comment.anchor?.startOffset).toBeGreaterThan(0)
  await expect(repeatedBlocks.nth(0).locator('.review-block__marker')).toHaveCount(0)
  await expect(secondBlock.locator('.review-block__marker')).not.toHaveCount(0)
})

test('surfaces server refusal when rendered selection crosses inline markup', async ({ page }) => {
  await page.goto(`/changes/${fixtureChange}`)
  const block = page
    .locator('.review-block')
    .filter({ hasText: 'Selection with inline markup crosses source.' })

  await block.locator('p').evaluate((paragraph) => {
    const range = document.createRange()
    range.selectNodeContents(paragraph)
    const selection = window.getSelection()
    selection?.removeAllRanges()
    selection?.addRange(range)
    paragraph.closest('.review-block')?.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))
  })
  await block.locator('..').getByLabel('Comment on selected text').fill('Markup must be explicit.')
  await block.locator('..').getByRole('button', { name: 'Record comment' }).click()

  await expect(page.getByRole('alert')).toContainText('was not found')
  await expect(page.getByRole('alert')).toContainText('Action not recorded')
})

test('replies, resolves, and reopens while second tab reconciles over events', async ({
  page,
  context,
}, testInfo) => {
  test.skip(
    testInfo.project.name !== 'desktop',
    'cross-tab mutation runs once against shared fixture',
  )
  const second = await context.newPage()
  await page.goto(`/changes/${fixtureChange}`)
  await second.goto(`/changes/${fixtureChange}`)

  const firstBlock = page
    .locator('.review-block')
    .filter({ hasText: 'Repeated review target.' })
    .nth(1)
  const secondBlock = second
    .locator('.review-block')
    .filter({ hasText: 'Repeated review target.' })
    .nth(1)
  await firstBlock.locator('.review-block__marker').first().click()
  await secondBlock.locator('.review-block__marker').first().click()

  await page.getByRole('button', { name: 'Reply' }).click()
  await page.getByLabel('Reply to thread').fill('Follow-up recorded from browser.')
  await page.getByRole('button', { name: 'Record reply' }).click()
  const browserReply = second
    .locator('.comment-message--reviewer')
    .filter({ hasText: 'Follow-up recorded from browser.' })
  await expect(browserReply.getByText('Reviewer')).toBeVisible()

  await page.getByRole('button', { name: 'Resolve' }).click()
  await expect(second.getByRole('button', { name: 'Reopen' })).toBeVisible()
  await page.getByRole('button', { name: 'Reopen' }).click()
  await expect(second.getByRole('button', { name: 'Resolve' })).toBeVisible()
})

test('submits empty composer as verdict without recording comment', async ({ page }) => {
  await page.goto(`/sessions/${fixtureSession}`)
  const before = await page.evaluate(
    async ({ session }) => {
      const response = await fetch(`/api/sessions/${session}`)
      if (!response.ok) throw new Error(`scope detail ${response.status}`)
      return ((await response.json()) as { comments: unknown[] }).comments.length
    },
    { session: fixtureSession },
  )

  await page.getByRole('button', { name: /Comment \/ 0 without block/ }).click()
  await page.getByRole('button', { name: 'Keep exploring' }).click()

  await expect(page.getByText('Not yet delivered')).toBeVisible()
  const after = await page.evaluate(
    async ({ session }) => {
      const response = await fetch(`/api/sessions/${session}`)
      if (!response.ok) throw new Error(`scope detail ${response.status}`)
      return ((await response.json()) as { comments: unknown[] }).comments.length
    },
    { session: fixtureSession },
  )
  expect(after).toBe(before)
})

test('comments on live exploration block with exact anchor', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'narrow', 'mutation runs after shared-state assertions')
  await page.goto(`/sessions/${fixtureSession}`)
  const block = page
    .locator('.review-block')
    .filter({ hasText: 'Review atmosphere must yield before content.' })

  await block.getByRole('button', { name: /Comment on .* block/ }).click()
  await block.locator('..').getByLabel('Comment on block').fill('Exploration block remains exact.')
  await block.locator('..').getByRole('button', { name: 'Record comment' }).click()

  const created = await page.evaluate(
    async ({ session }) => {
      const response = await fetch(`/api/sessions/${session}`)
      if (!response.ok) throw new Error(`scope detail ${response.status}`)
      const detail = (await response.json()) as {
        comments: Array<{ comment: { body: string }; anchorState: string }>
      }
      return detail.comments.find(
        (thread) => thread.comment.body === 'Exploration block remains exact.',
      )
    },
    { session: fixtureSession },
  )
  expect(created?.anchorState).toBe('exact')
})

test('keeps scope document, expanded thread, controls, and atmosphere in narrow flow', async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'narrow', '390px contract')
  await page.goto(`/changes/${fixtureChange}`)
  const repeatedBlock = page
    .locator('.review-block')
    .filter({ hasText: 'Repeated review target.' })
    .nth(1)
  await repeatedBlock.locator('.review-block__marker').first().click()
  const conversation = repeatedBlock.locator('..').locator('.review-block-row__conversation')

  const layout = await page.evaluate(() => ({
    viewport: window.innerWidth,
    documentWidth: document.documentElement.scrollWidth,
    artwork: getComputedStyle(document.querySelector('.scope-header .orbital-frame') as Element)
      .display,
  }))
  expect(layout.documentWidth).toBeLessThanOrEqual(layout.viewport)
  expect(layout.artwork).toBe('none')
  expect((await conversation.boundingBox())?.y).toBeGreaterThanOrEqual(
    (await repeatedBlock.boundingBox())?.y ?? 0,
  )
  await expect(page.getByRole('button', { name: /Comment \/ .* without block/ })).toBeVisible()
})

test('captures embedded observatory index for source-board review', async ({ page }, testInfo) => {
  const health = observeBrowserHealth(page)
  await page.goto('/')
  await expect(page.getByRole('link', { name: fixtureTitle })).toBeVisible()

  await page.screenshot({
    path: testInfo.outputPath(`observatory-index-${testInfo.project.name}.png`),
    fullPage: true,
  })
  expectHealthy(health)
})
