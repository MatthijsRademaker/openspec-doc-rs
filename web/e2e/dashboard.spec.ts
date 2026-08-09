import { expect, type Page, test } from '@playwright/test'
import { writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const fixtureChange = 'implement-observatory-design-system-with-a-realistically-long-identifier'
const fixtureTitle = 'Observatory Design System Fixture'
const fixtureSession = '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74'
const fixtureSessionTitle = 'Agent-guided observatory exploration session'
const untitledFixtureSession = '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d77'
const fixtureScopeCount = { changes: 3, sessions: 9 }
const changeRoot = `openspec/changes/${fixtureChange}`
const proposalPath = `${changeRoot}/proposal.md`
const designPath = `${changeRoot}/design.md`
const tasksPath = `${changeRoot}/tasks.md`
const htmlSpecPath = `${changeRoot}/specs/dashboard-html-views/spec.md`
const visualSpecPath = `${changeRoot}/specs/dashboard-visual-system/spec.md`
const fixturePaths = [proposalPath, designPath, tasksPath, htmlSpecPath, visualSpecPath]
const repeatedBlock = 'Repeated review target.'
const proposalSource =
  '# Observatory Design System Fixture\n\nBrowser lane must see this proposal.\n\n' +
  `${repeatedBlock}\n\nA bridge between repeated blocks.\n\n${repeatedBlock}\n\n` +
  'Selection with **inline markup** crosses source.\n'

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
    if (message.type() === 'error') health.consoleErrors.push(message.text())
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

function artifactButton(page: Page, localPath: string) {
  return page.locator('.artifact-navigator__path').filter({ hasText: localPath })
}

async function expectArtifactQuery(page: Page, path: string): Promise<void> {
  await expect(page).toHaveURL((url) => url.searchParams.get('artifact') === path)
}

async function gotoArtifact(page: Page, path: string): Promise<void> {
  await page.goto(`/changes/${fixtureChange}?${new URLSearchParams({ artifact: path })}`)
  await expectArtifactQuery(page, path)
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

test('keeps sole theme and all font roles offline', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('openspec-doc-theme', 'light'))
  const health = observeBrowserHealth(page)
  await page.goto('/')
  await page.evaluate(() => document.fonts.ready)

  await expect(page.getByRole('button', { name: /theme/i })).toHaveCount(0)
  expect(
    await page.evaluate(() => ({
      background: getComputedStyle(document.body).backgroundColor,
      displayLoaded: document.fonts.check('16px "Cormorant Garamond"'),
      proseLoaded: document.fonts.check('16px "IBM Plex Sans Variable"'),
      monoLoaded: document.fonts.check('16px "IBM Plex Mono"'),
    })),
  ).toEqual({
    background: 'rgb(11, 12, 14)',
    displayLoaded: true,
    proseLoaded: true,
    monoLoaded: true,
  })
  expectHealthy(health)
})

test('canonicalizes default artifact, restores copied nested URL, and supports Back Forward', async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'desktop Router contract')
  await page.goto(`/changes/${fixtureChange}`)

  await expectArtifactQuery(page, proposalPath)
  await expect(page.getByText('Browser lane must see this proposal.')).toBeVisible()
  await expect(page.getByText('Conversation follows exact selected artifact.')).toHaveCount(0)
  await expect(page.locator('.artifact-navigator__path')).toHaveCount(6)
  for (const path of fixturePaths) {
    await expect(artifactButton(page, path.slice(changeRoot.length + 1))).toBeVisible()
  }

  await artifactButton(page, 'design.md').click()
  await expectArtifactQuery(page, designPath)
  await expect(page.getByText('Conversation follows exact selected artifact.')).toBeVisible()
  await expect(page.getByText('Browser lane must see this proposal.')).toHaveCount(0)

  await artifactButton(page, 'tasks.md').click()
  await expectArtifactQuery(page, tasksPath)
  await expect(page.getByText('Render index data')).toBeVisible()

  await page.goBack()
  await expectArtifactQuery(page, designPath)
  await expect(page.getByText('Conversation follows exact selected artifact.')).toBeVisible()
  await page.goForward()
  await expectArtifactQuery(page, tasksPath)

  await gotoArtifact(page, htmlSpecPath)
  await expect(page.getByText('Nested paths remain exact.')).toBeVisible()
  await page.reload()
  await expect(page.getByText('Nested paths remain exact.')).toBeVisible()
})

test('keeps invalid exact artifact visible without fallback', async ({ page }) => {
  const health = observeBrowserHealth(page)
  const missing = `${changeRoot}/specs/missing-capability/spec.md`
  await gotoArtifact(page, missing)

  await expect(page.getByRole('heading', { name: 'Selected artifact unavailable' })).toBeVisible()
  await expect(page.getByText(missing)).toBeVisible()
  await expect(page.locator('.artifact-navigator__path')).toHaveCount(6)
  await expect(page.getByText('Browser lane must see this proposal.')).toHaveCount(0)
  await expect(page.locator('.scope-header h1')).toHaveText(fixtureTitle)
  expectHealthy(health)
})

test('auto-selects sole session scratch without fake artifact navigation', async ({ page }) => {
  const health = observeBrowserHealth(page)
  await page.goto(`/sessions/${fixtureSession}`)

  await expect(page.getByRole('heading', { name: 'Exploration scratch' })).toBeVisible()
  await expect(page.getByText('Review atmosphere must yield before content.')).toBeVisible()
  await expect(page.locator('.artifact-navigator--identity')).toBeVisible()
  await expect(page.getByRole('navigation', { name: 'Artifacts' })).toHaveCount(0)
  expectHealthy(health)
})

test('composes full-bleed selected-document chassis without obsolete gallery or overlap', async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'desktop composition contract')
  const health = observeBrowserHealth(page)
  const artworkRequests: string[] = []
  page.on('request', (request) => {
    const pathname = parseUrl(request.url(), 'artwork request').pathname
    if (pathname.startsWith('/assets/images/observatory-')) artworkRequests.push(pathname)
  })
  await gotoArtifact(page, proposalPath)

  await expect(page.getByText('Browser lane must see this proposal.')).toBeInViewport()
  await expect(page.locator('.artifact-document__arrival-art img')).toHaveAttribute('alt', '')
  await expect(page.locator('.artifact-document__arrival-art')).toHaveAttribute(
    'aria-hidden',
    'true',
  )
  await expect(page.locator('.scope-observation-band')).toHaveCount(0)
  expect([...new Set(artworkRequests)]).toEqual(['/assets/images/observatory-field.webp'])
  await expect(page.getByText('Keep repeated occurrence mapping exact.')).toBeVisible()
  await expect(page.getByText('Keep design conversation artifact-scoped.')).toHaveCount(0)
  await expect(page.locator('body')).not.toContainText(
    /repository|activity feed|validation result/i,
  )

  const geometry = await page.evaluate(() => {
    const utility = document.querySelector('.scope-utility')?.getBoundingClientRect()
    const documentStage = document.querySelector('.scope-document')?.getBoundingClientRect()
    const conversation = document.querySelector('.scope-conversation')?.getBoundingClientRect()
    const decision = document.querySelector('.decision-instrument__bar')?.getBoundingClientRect()
    return {
      utility: utility?.toJSON(),
      documentStage: documentStage?.toJSON(),
      conversation: conversation?.toJSON(),
      decision: decision?.toJSON(),
      bodyOverflow: getComputedStyle(document.body).overflowY,
      documentOverflow: document.querySelector('.scope-document')
        ? getComputedStyle(document.querySelector('.scope-document') as Element).overflowY
        : null,
    }
  })
  expect(geometry.utility?.right).toBeLessThanOrEqual(geometry.documentStage?.left ?? 0)
  expect(geometry.documentStage?.right).toBeLessThanOrEqual(geometry.conversation?.left ?? 0)
  expect(geometry.decision?.left).toBeGreaterThanOrEqual(geometry.documentStage?.right ?? 0)
  expect(geometry.documentOverflow).not.toBe('scroll')
  expect(geometry.documentOverflow).not.toBe('auto')
  expectHealthy(health)
})

test('links persistent conversation to exact repeated source occurrence', async ({ page }) => {
  await gotoArtifact(page, proposalPath)
  const repeatedBlocks = page
    .locator('.review-block')
    .filter({ hasText: 'Repeated review target.' })
  await expect(repeatedBlocks).toHaveCount(2)
  await expect(repeatedBlocks.nth(0).locator('.review-block__marker')).toHaveCount(0)
  await expect(repeatedBlocks.nth(1).locator('.review-block__marker')).toHaveCount(1)

  await repeatedBlocks.nth(1).locator('.review-block__marker').click()
  await expect(repeatedBlocks.nth(1)).toHaveClass(/review-block--active/)
  await expect(page.locator('#artifact-thread-e2e-open-comment')).toHaveClass(
    /artifact-conversation__thread--active/,
  )
  await expect(page.locator('#artifact-thread-e2e-open-comment')).toBeFocused()

  await page.getByRole('button', { name: 'Locate source for comment e2e-open-comment' }).click()
  await expect(repeatedBlocks.nth(1)).toBeFocused()
  await expect(page.getByText('Reviewer', { exact: true }).first()).toBeVisible()
  await expect(page.getByText('Agent', { exact: true }).first()).toBeVisible()
  await expect(page.getByRole('button', { name: 'Reply' })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Resolve', exact: true })).toBeVisible()
})

test('anchors new comment to second repeated block and refuses inline-markup mismatch', async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'narrow', 'mutation runs once after shared assertions')
  await gotoArtifact(page, proposalPath)
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

  const inlineBlock = page
    .locator('.review-block')
    .filter({ hasText: 'Selection with inline markup crosses source.' })
  await inlineBlock.locator('p').evaluate((paragraph) => {
    const range = document.createRange()
    range.selectNodeContents(paragraph)
    const selection = window.getSelection()
    selection?.removeAllRanges()
    selection?.addRange(range)
    paragraph.closest('.review-block')?.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))
  })
  await inlineBlock
    .locator('..')
    .getByLabel('Comment on selected text')
    .fill('Markup must be explicit.')
  await inlineBlock.locator('..').getByRole('button', { name: 'Record comment' }).click()
  await expect(page.getByRole('alert')).toContainText('was not found')
  await expect(page.getByRole('alert')).toContainText('Action not recorded')
})

test('replies, resolves, and reopens while second tab reconciles selected thread', async ({
  page,
  context,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'cross-tab mutation runs once')
  const second = await context.newPage()
  await gotoArtifact(page, proposalPath)
  await gotoArtifact(second, proposalPath)

  await page.getByRole('button', { name: 'Reply' }).click()
  await page.getByLabel('Reply to thread').fill('Follow-up recorded from browser.')
  await page.getByRole('button', { name: 'Record reply' }).click()
  await expect(
    second
      .locator('.comment-message--reviewer')
      .filter({ hasText: 'Follow-up recorded from browser.' }),
  ).toBeVisible()

  await page.getByRole('button', { name: 'Resolve', exact: true }).click()
  await expect(second.getByRole('button', { name: 'Reopen' }).first()).toBeVisible()
  await page.getByRole('button', { name: 'Reopen' }).first().click()
  await expect(second.getByRole('button', { name: 'Resolve', exact: true })).toBeVisible()
  await second.close()
})

test('keeps dirty composer while review state updates and applies latest deferred artifact', async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'filesystem live-update contract runs once')
  const fixtureRoot = process.env.E2E_FIXTURE_ROOT
  if (!fixtureRoot) throw new Error('E2E_FIXTURE_ROOT is required for live artifact verification')
  const proposalFile = join(fixtureRoot, proposalPath)
  await gotoArtifact(page, proposalPath)

  const sourceBlock = page.locator('.review-block').filter({
    hasText: 'Browser lane must see this proposal.',
  })
  await sourceBlock.getByRole('button', { name: /Comment on .* block/ }).click()
  await sourceBlock.locator('..').getByLabel('Comment on block').fill('Keep dirty composer text.')

  await page.evaluate(
    async ({ change }) => {
      const response = await fetch(`/api/changes/${change}/comments/e2e-open-comment/status`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ status: 'resolved' }),
      })
      if (!response.ok) throw new Error(`resolve comment ${response.status}`)
    },
    { change: fixtureChange },
  )
  await expect(page.getByText('2 resolved', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Comment on block')).toHaveValue('Keep dirty composer text.')

  const firstRewrite = proposalSource.replace(
    'Browser lane must see this proposal.',
    'First live rewrite.',
  )
  const latestRewrite = proposalSource.replace(
    'Browser lane must see this proposal.',
    'Latest live rewrite.',
  )
  await writeFile(proposalFile, firstRewrite)
  await expect(page.getByText('Artifact changed.')).toBeVisible()
  await writeFile(proposalFile, latestRewrite)
  await expect(page.getByText('Latest live rewrite.')).toHaveCount(0)
  await expect(page.getByLabel('Comment on block')).toHaveValue('Keep dirty composer text.')

  await page.getByRole('button', { name: 'Cancel' }).click()
  await expect(page.getByText('Latest live rewrite.')).toBeVisible()
  await expectArtifactQuery(page, proposalPath)

  await page.evaluate(
    async ({ change }) => {
      const response = await fetch(`/api/changes/${change}/comments/e2e-open-comment/status`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ status: 'open' }),
      })
      if (!response.ok) throw new Error(`reopen comment ${response.status}`)
    },
    { change: fixtureChange },
  )
  await writeFile(proposalFile, proposalSource)
  await page.reload()
  await expect(page.getByText('Browser lane must see this proposal.')).toBeVisible()
})

test('keeps orphaned and addressed loose comments plus verdict actions', async ({ page }) => {
  await gotoArtifact(page, proposalPath)
  await expect(page.getByText('Not yet delivered')).toBeVisible()
  await page.getByRole('button', { name: /Comment \/ 2 without block/ }).click()
  await expect(page.getByText('Lost anchors must stay reachable.')).toBeVisible()
  await expect(page.getByText('Anchor lost')).toBeVisible()
  await expect(page.getByText('Original text rewritten away.')).toBeVisible()
  await expect(page.getByText(/Agent claims work complete/)).toBeVisible()
  await expect(page.getByRole('button', { name: 'Accept as resolved' })).toBeVisible()

  await page.goto(`/sessions/${fixtureSession}`)
  const before = await page.evaluate(async (session) => {
    const response = await fetch(`/api/sessions/${session}`)
    return ((await response.json()) as { comments: unknown[] }).comments.length
  }, fixtureSession)
  await page.getByRole('button', { name: /Comment \/ 0 without block/ }).click()
  await page.getByRole('button', { name: 'Keep exploring' }).click()
  const after = await page.evaluate(async (session) => {
    const response = await fetch(`/api/sessions/${session}`)
    return ((await response.json()) as { comments: unknown[] }).comments.length
  }, fixtureSession)
  expect(after).toBe(before)
  await expect(page.getByText('Not yet delivered')).toBeVisible()
  await expect(page.getByRole('button', { name: 'Move to proposal' })).toBeVisible()
})

test('keeps selected-artifact workbench in one complete narrow flow', async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'narrow', '390×844 contract')
  await gotoArtifact(page, visualSpecPath)
  await expect(page.getByText('Artwork yields before prose.')).toBeVisible()
  await expect(page.locator('.artifact-document__arrival-art')).toBeHidden()
  await expect(page.locator('.scope-observation-band')).toHaveCount(0)

  const order = await page.evaluate(() => {
    const utility = document.querySelector('.scope-utility')
    const documentStage = document.querySelector('.scope-document')
    const conversation = document.querySelector('.scope-conversation')
    const relation = (left: Element | null, right: Element | null) =>
      Boolean(
        left && right && left.compareDocumentPosition(right) & Node.DOCUMENT_POSITION_FOLLOWING,
      )
    return {
      viewport: window.innerWidth,
      documentWidth: document.documentElement.scrollWidth,
      utilityBeforeDocument: relation(utility, documentStage),
      documentBeforeConversation: relation(documentStage, conversation),
      navigatorWidth: document.querySelector('.artifact-navigator')?.getBoundingClientRect().width,
      finalBlockBottom: document
        .querySelector('.review-block-row:last-child')
        ?.getBoundingClientRect().bottom,
    }
  })
  expect(order.documentWidth).toBeLessThanOrEqual(order.viewport)
  expect(order.utilityBeforeDocument).toBe(true)
  expect(order.documentBeforeConversation).toBe(true)
  expect(order.navigatorWidth).toBeLessThanOrEqual(order.viewport)

  await artifactButton(page, 'proposal.md').click()
  const repeatedBlock = page
    .locator('.review-block')
    .filter({ hasText: 'Repeated review target.' })
    .nth(1)
  await expect(repeatedBlock.getByRole('button', { name: /Comment on .* block/ })).toBeVisible()
  await repeatedBlock.locator('.review-block__marker').first().click()
  await expect(page.locator('#artifact-thread-e2e-open-comment')).toBeFocused()
  await page.getByRole('button', { name: 'Locate source for comment e2e-open-comment' }).click()
  await expect(repeatedBlock).toBeFocused()

  await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight))
  const clearance = await page.evaluate(() => ({
    viewportBottom: window.innerHeight,
    lastContentBottom: document.querySelector('.scope-conversation')?.getBoundingClientRect()
      .bottom,
  }))
  expect(clearance.lastContentBottom).toBeLessThanOrEqual(clearance.viewportBottom)
})

test('keeps index identifiers and every session reachable in narrow flow', async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== 'narrow', '390px index contract')
  await page.goto('/')
  for (const value of [
    fixtureChange,
    fixtureSession,
    untitledFixtureSession,
    'comment-resolution',
    'most recently active',
  ]) {
    await expect(page.getByText(value).first()).toBeVisible()
  }
  await expect(page.getByText(/^\d+ open$/).first()).toBeVisible()
  await expect(page.locator('.index-workbench a')).toHaveCount(
    fixtureScopeCount.changes + fixtureScopeCount.sessions,
  )
  expect(
    await page.evaluate(() => ({
      viewport: window.innerWidth,
      documentWidth: document.documentElement.scrollWidth,
      platesDisplay: getComputedStyle(document.querySelector('.index-plates') as Element).display,
    })),
  ).toEqual({ viewport: 390, documentWidth: 390, platesDisplay: 'none' })
})

test('keeps keyboard focus and reduced-motion navigation immediate', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'desktop keyboard inspection')
  await page.emulateMedia({ reducedMotion: 'reduce' })
  await gotoArtifact(page, proposalPath)

  const focusTrail: string[] = []
  for (let step = 0; step < 40; step += 1) {
    await page.keyboard.press('Tab')
    focusTrail.push(
      await page.evaluate(
        () => document.activeElement?.textContent?.replace(/\s+/g, ' ').trim() ?? '',
      ),
    )
  }
  expect(focusTrail.some((text) => text.includes('Verdict history / 1'))).toBe(true)
  expect(focusTrail.some((text) => text.includes('Observation index'))).toBe(true)
  expect(focusTrail.some((text) => text.includes('proposal.md'))).toBe(true)
  expect(focusTrail.some((text) => text.includes('open comment'))).toBe(true)
  expect(focusTrail.some((text) => text.includes('Comment / 2 without block'))).toBe(true)

  const selectedPath = page.locator('.artifact-navigator__path[aria-current="page"]')
  await selectedPath.focus()
  await expect(selectedPath).toBeFocused()
  expect(
    await selectedPath.evaluate((element) => {
      const style = getComputedStyle(element)
      return { outline: style.outlineStyle, duration: style.transitionDuration }
    }),
  ).toEqual({ outline: 'solid', duration: '0s' })

  await page.getByRole('button', { name: 'Locate source for comment e2e-open-comment' }).click()
  const repeatedBlock = page
    .locator('.review-block')
    .filter({ hasText: 'Repeated review target.' })
    .nth(1)
  await expect(repeatedBlock).toBeFocused()
})

test('captures deterministic selected-artifact design evidence', async ({ page }, testInfo) => {
  const health = observeBrowserHealth(page)
  await gotoArtifact(page, proposalPath)
  await expect(page.getByText('Browser lane must see this proposal.')).toBeVisible()

  await page.screenshot({
    path: testInfo.outputPath(`observatory-scope-${testInfo.project.name}.png`),
    fullPage: false,
  })
  if (testInfo.project.name === 'desktop') {
    await page.setViewportSize({ width: 1536, height: 1024 })
    await page.screenshot({
      path: testInfo.outputPath('observatory-scope-target-1536x1024.png'),
      fullPage: false,
    })
  }
  expectHealthy(health)
})
