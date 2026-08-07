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
const fixtureChange = 'e2e-dashboard-fixture'
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

  const files: Record<string, string> = {
    'openspec/config.yaml': '',
    [`openspec/changes/${fixtureChange}/.openspec.yaml`]:
      'schema: spec-driven\ncreated: 2026-01-01\n',
    [`openspec/changes/${fixtureChange}/proposal.md`]:
      '# E2E Dashboard Fixture\n\nBrowser lane must see this proposal.\n',
    [`openspec/changes/${fixtureChange}/design.md`]:
      '# Fixture design\n\nA deterministic embedded-app fixture.\n',
    [`openspec/changes/${fixtureChange}/tasks.md`]: '# Fixture tasks\n\n- [x] Render index data\n',
    [`openspec/changes/${fixtureChange}/specs/index/spec.md`]:
      '# Index fixture\n\nThe fixture has one active change.\n',
    [`.openspec-doc/scratch/${fixtureChange}.md`]:
      '# E2E Dashboard Fixture\n\nThis title is intentionally long enough to exercise a real row.\n',
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
