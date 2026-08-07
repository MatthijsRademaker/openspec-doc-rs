import { type ChildProcess, spawn } from 'node:child_process'
import { mkdir, mkdtemp, rm, utimes, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const moduleRoot = fileURLToPath(new URL('.', import.meta.url))
const repoRoot = resolve(moduleRoot, '../..')
const webRoot = resolve(moduleRoot, '..')
const port = 8792
const baseURL = `http://127.0.0.1:${port}`
const fixtureChange = 'implement-observatory-design-system-with-a-realistically-long-identifier'
const fixtureSession = '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74'
const bun = process.execPath
const binary = join(repoRoot, 'target', 'debug', 'openspec-doc')

function run(command: string, args: string[], cwd: string): Promise<void> {
  return new Promise((resolveRun, rejectRun) => {
    const child = spawn(command, args, { cwd, stdio: 'inherit' })
    child.once('error', rejectRun)
    child.once('exit', (code, signal) => {
      if (code === 0) {
        resolveRun()
        return
      }
      rejectRun(new Error(`${command} ${args.join(' ')} exited with ${code ?? signal}`))
    })
  })
}

function waitForExit(child: ChildProcess, timeoutMs?: number): Promise<boolean> {
  if (child.exitCode !== null) {
    return Promise.resolve(true)
  }
  return new Promise((resolveExit) => {
    const timer = timeoutMs ? setTimeout(() => resolveExit(false), timeoutMs) : undefined
    child.once('exit', () => {
      if (timer) {
        clearTimeout(timer)
      }
      resolveExit(true)
    })
  })
}

async function stop(child: ChildProcess): Promise<void> {
  if (child.exitCode !== null) {
    return
  }
  child.kill('SIGTERM')
  const exited = await waitForExit(child, 2_000)
  if (!exited && child.exitCode === null) {
    child.kill('SIGKILL')
    await waitForExit(child)
  }
}

async function createFixture(): Promise<string> {
  const root = await mkdtemp(join(tmpdir(), 'openspec-doc-e2e-'))
  const changeRoot = join(root, 'openspec', 'changes', fixtureChange)
  const scratchRoot = join(root, '.openspec-doc', 'scratch')
  await mkdir(changeRoot, { recursive: true })
  await mkdir(scratchRoot, { recursive: true })

  const repeatedBlock = 'Repeated review target.'
  const proposal =
    '# Observatory Design System Fixture\n\nBrowser lane must see this proposal.\n\n' +
    `${repeatedBlock}\n\nA bridge between repeated blocks.\n\n${repeatedBlock}\n\n` +
    'Selection with **inline markup** crosses source.\n'
  const secondRepeatedOffset = proposal.lastIndexOf(repeatedBlock)
  const comments = [
    {
      type: 'comment',
      comment: {
        id: 'e2e-open-comment',
        anchor: {
          artifactPath: `openspec/changes/${fixtureChange}/proposal.md`,
          selectedText: repeatedBlock,
          headingPath: ['Observatory Design System Fixture'],
          beforeText: '',
          afterText: '',
          startOffset: secondRepeatedOffset,
          endOffset: secondRepeatedOffset + repeatedBlock.length,
        },
        body: 'Keep repeated occurrence mapping exact.',
        createdAt: '2026-01-01T00:00:00Z',
      },
    },
    {
      type: 'reply',
      reply: {
        id: 'e2e-agent-reply',
        commentId: 'e2e-open-comment',
        author: 'agent',
        body: 'Second occurrence is now explicit.',
        createdAt: '2026-01-01T00:01:00Z',
      },
    },
    {
      type: 'comment',
      comment: {
        id: 'e2e-orphaned-comment',
        anchor: {
          artifactPath: `openspec/changes/${fixtureChange}/proposal.md`,
          selectedText: 'Original text rewritten away.',
          headingPath: [],
          beforeText: '',
          afterText: '',
          startOffset: 900,
          endOffset: 929,
        },
        body: 'Lost anchors must stay reachable.',
        createdAt: '2026-01-01T00:02:00Z',
      },
    },
    {
      type: 'status',
      status: {
        id: 'e2e-orphan-resolved',
        commentId: 'e2e-orphaned-comment',
        status: 'resolved',
        createdAt: '2026-01-01T00:03:00Z',
      },
    },
    {
      type: 'comment',
      comment: {
        id: 'e2e-unanchored-comment',
        anchor: null,
        body: 'Whole-scope agent claim.',
        createdAt: '2026-01-01T00:04:00Z',
      },
    },
    {
      type: 'status',
      status: {
        id: 'e2e-unanchored-addressed',
        commentId: 'e2e-unanchored-comment',
        status: 'addressed',
        createdAt: '2026-01-01T00:05:00Z',
      },
    },
  ]
    .map((event) => JSON.stringify(event))
    .join('\n')

  const files: Record<string, string> = {
    'openspec/config.yaml': '',
    [`openspec/changes/${fixtureChange}/.openspec.yaml`]:
      'schema: spec-driven\ncreated: 2026-01-01\n',
    [`openspec/changes/${fixtureChange}/proposal.md`]: proposal,
    [`openspec/changes/${fixtureChange}/design.md`]:
      '# Fixture design\n\nA deterministic embedded-app fixture.\n',
    [`openspec/changes/${fixtureChange}/tasks.md`]: '# Fixture tasks\n\n- [x] Render index data\n',
    [`openspec/changes/${fixtureChange}/specs/index/spec.md`]:
      '# Index fixture\n\nThe fixture has one active change.\n',
    [`.openspec-doc/scratch/${fixtureChange}.md`]:
      '# Observatory Design System Fixture\n\nThis title exercises a real ruled register.\n',
    [`.openspec-doc/directives/_session/${fixtureSession}.json`]:
      '{"pending":false,"reason":"none","createdAt":"2026-01-01T00:00:00Z","consumedAt":null}\n',
    [`.openspec-doc/scratch/_session/${fixtureSession}.md`]:
      '# Agent-guided observatory exploration session\n\nReview atmosphere must yield before content.\n',
    [`.openspec-doc/comments/${fixtureChange}.jsonl`]: `${comments}\n`,
    [`.openspec-doc/verdicts/${fixtureChange}.jsonl`]:
      '{"id":"e2e-verdict","verdict":"comment-resolution","notes":"","createdAt":"2026-01-01T00:00:01Z"}\n',
  }
  const fixedTime = new Date('2026-01-01T00:00:00Z')
  for (const [relative, contents] of Object.entries(files)) {
    const path = join(root, relative)
    await mkdir(dirname(path), { recursive: true })
    await writeFile(path, contents)
    await utimes(path, fixedTime, fixedTime)
  }
  return root
}

async function waitForServer(server: ChildProcess, output: string[]): Promise<void> {
  let lastError: unknown
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (server.exitCode !== null) {
      throw new Error(`openspec-doc exited before readiness:\n${output.join('')}`)
    }
    try {
      // pi-lens-ignore: typescript.react.security.react-insecure-request.react-insecure-request
      const response = await fetch(`${baseURL}/api/index`)
      if (response.ok) {
        return
      }
      lastError = new Error(`HTTP ${response.status}`)
    } catch (error) {
      lastError = error
    }
    await new Promise((resolveSleep) => setTimeout(resolveSleep, 100))
  }
  throw new Error(
    `Timed out waiting for ${baseURL}/api/index: ${String(lastError)}\n${output.join('')}`,
  )
}

async function main(): Promise<void> {
  let fixture: string | undefined
  let server: ChildProcess | undefined
  let failure: unknown
  try {
    await run(bun, ['run', 'build'], webRoot)
    await run('cargo', ['build', '--locked', '--package', 'openspec-doc-cli'], repoRoot)
    fixture = await createFixture()

    const output: string[] = []
    server = spawn(
      binary,
      ['--root', fixture, 'serve', '--host', '127.0.0.1', '--port', String(port), '--no-open'],
      { cwd: repoRoot, stdio: ['ignore', 'pipe', 'pipe'] },
    )
    server.stdout?.on('data', (chunk: Buffer) => output.push(chunk.toString()))
    server.stderr?.on('data', (chunk: Buffer) => output.push(chunk.toString()))
    await waitForServer(server, output)

    await run(
      bun,
      [
        join(webRoot, 'node_modules', '@playwright', 'test', 'cli.js'),
        'test',
        '--config',
        join(webRoot, 'playwright.config.ts'),
      ],
      webRoot,
    )
  } catch (error) {
    failure = error
  } finally {
    if (server) {
      await stop(server)
    }
    if (fixture) {
      await rm(fixture, { recursive: true, force: true })
    }
  }
  if (failure) {
    throw failure
  }
}

await main()
