// The index's data as the server serves it. Every field here is read from the
// scratch, comment and verdict sidecars — see `crates/server/src/api.rs`.

export type Verdict = 'keep-exploring' | 'move-to-proposal' | 'comment-resolution'

export interface Scope {
  /** The session id or change name. It is the scope's identity and its address;
   *  the title is display text and never either. */
  key: string
  /** What the scope is called, when it is called anything but its key. */
  title: string | null
  /** RFC 3339, or null while none of the scope's artifacts exists. */
  modifiedAt: string | null
  openComments: number
  verdict: Verdict | null
  mostRecentlyActive: boolean
}

export interface Index {
  sessions: Scope[]
  changes: Scope[]
}

export async function fetchIndex(): Promise<Index> {
  const response = await fetch('/api/index')
  if (!response.ok) {
    throw new Error(`GET /api/index responded ${response.status} ${response.statusText}`)
  }

  return response.json() as Promise<Index>
}
