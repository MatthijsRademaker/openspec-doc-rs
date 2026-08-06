## ADDED Requirements

### Requirement: A running dashboard is discovered by probing, not by reading recorded state
The system SHALL determine whether a dashboard is already serving a project root by probing a bounded range of local ports for an identity response, and SHALL NOT depend on any recorded file naming the running dashboard's port or process.

Runtime state recorded under `.openspec-doc/` has to be excluded from version control, which means `git clean -xdf` and `rm -rf .openspec-doc` delete it while the server it describes keeps running. Every such deletion would then produce another dashboard that is alive, undiscoverable, and unkillable by any command this project could offer. Probing has no equivalent failure: it survives file deletion, reboot, and `SIGKILL`.

#### Scenario: A dashboard already serving this root is reused
- **WHEN** the system is asked to ensure a dashboard for a project root and a dashboard for that same canonical root answers on a probed port
- **THEN** the system SHALL reuse it and SHALL NOT start another

#### Scenario: A dashboard serving a different root is not mistaken for this one
- **WHEN** a probed port answers with an identity naming a different canonical project root
- **THEN** the system SHALL leave that dashboard alone and continue probing

#### Scenario: An unrelated service on a probed port is not mistaken for a dashboard
- **WHEN** a probed port accepts a connection but does not answer with a dashboard identity
- **THEN** the system SHALL continue probing and SHALL NOT treat that port as a dashboard

#### Scenario: Deleted runtime state does not produce a duplicate
- **WHEN** everything under `.openspec-doc/` is deleted while a dashboard for that root is running, and the system is then asked to ensure a dashboard
- **THEN** the system SHALL discover the running dashboard and SHALL NOT start a second one

### Requirement: The dashboard reports the root it serves and its process
The system SHALL expose, on the dashboard's HTTP surface, a route reporting the canonical project root that dashboard is serving and its process id.

Discovery compares canonical roots, so the comparison is exact rather than heuristic. The process id is reported because it is the only thing that makes an unwanted dashboard killable once discovery has found it.

#### Scenario: The identity route names the canonical root
- **WHEN** the identity route of a dashboard serving a project root is requested
- **THEN** the response SHALL contain that root in canonical form and the dashboard's process id

### Requirement: A started dashboard outlives the process that started it
The system SHALL start the dashboard detached from the starting process, in its own process group, so that the dashboard survives the exit or termination of the hook invocation that started it.

An agent may terminate a hook that overruns its timeout, and a child sharing the hook's process group is terminated with it. A dashboard that does not survive this works when tested by hand and never survives a real hook invocation.

#### Scenario: The dashboard survives its starter exiting
- **WHEN** a hook invocation starts a dashboard and then exits
- **THEN** the dashboard SHALL still be serving

#### Scenario: The dashboard survives its starter's process group being signalled
- **WHEN** the process group of the hook invocation that started a dashboard is terminated
- **THEN** the dashboard SHALL still be serving

### Requirement: A dashboard that fails to start is reported, not swallowed
The system SHALL direct a started dashboard's standard output and standard error to a log file under `.openspec-doc/`, SHALL wait a bounded time for the started dashboard to answer its identity route, and SHALL report a start that does not answer within that time, naming the log.

A start whose output was discarded and whose failure went unreported is a start that appears to have succeeded. The next invocation would repeat it identically and forever, which is the silent-failure shape this project forbids.

#### Scenario: A dashboard that comes up is reported with its URL
- **WHEN** a dashboard is started and answers its identity route within the wait
- **THEN** the system SHALL report the URL it is serving on

#### Scenario: A dashboard that never comes up fails loudly
- **WHEN** a dashboard is started and does not answer its identity route within the wait
- **THEN** the system SHALL report that the dashboard did not start and SHALL name the log file holding its output

#### Scenario: The started dashboard's output is captured
- **WHEN** a dashboard is started by a hook
- **THEN** its standard output and standard error SHALL be written to a log file under `.openspec-doc/`

### Requirement: Simultaneous starts converge on one dashboard
The system SHALL treat the presence of a dashboard for its root on the intended port as success, whether or not the dashboard was started by this invocation.

Two sessions reaching a turn boundary at the same moment both find the same port free and both start a dashboard. One loses the bind and exits; the other serves. Checking that a dashboard is there, rather than that our own child is the one that put it there, makes the loser of that race a non-event instead of an error path.

#### Scenario: The invocation that lost the bind still ends with a dashboard
- **WHEN** two invocations start a dashboard for the same root concurrently and one fails to bind
- **THEN** both invocations SHALL report a dashboard serving that root, and exactly one dashboard SHALL be serving it

### Requirement: A dashboard is ensured only for sessions that have something to review
The system SHALL ensure a dashboard at a turn boundary only when the session has review material — a scratch note, or a change its exploration was promoted to — and SHALL NOT ensure one for a session that has neither.

Every turn boundary in the project runs the turn-end hook, including sessions doing work that has nothing to do with review. Those sessions must not leave a dashboard behind. The note's existence is already the project's marker that an exploration actually started, and promotion already depends on it.

#### Scenario: A session with an exploration keeps a dashboard up
- **WHEN** a turn boundary is reached in a session whose scratch note exists
- **THEN** the system SHALL ensure a dashboard is serving the project root

#### Scenario: A session with no exploration starts no dashboard
- **WHEN** a turn boundary is reached in a session with no scratch note and no promoted change
- **THEN** the system SHALL NOT start a dashboard

#### Scenario: A promoted exploration keeps a dashboard up
- **WHEN** a turn boundary is reached in a session whose exploration has been promoted to a change
- **THEN** the system SHALL ensure a dashboard is serving the project root

### Requirement: Turn boundaries keep the dashboard alive and restore it
Each ensure at a turn boundary SHALL count as activity against the dashboard's idle deadline, and SHALL start a dashboard when none is serving the root.

The reviewer's attention is asynchronous — that is why feedback is routed through a turn boundary rather than a chat message — so an open page is not evidence that the dashboard is wanted, and a closed one is not evidence that it is not. A hook that already fires every turn is the liveness signal the idle deadline needs, and it costs nothing because the same probe decides whether to start one.

#### Scenario: An active session holds the dashboard open with no page subscribed
- **WHEN** turn boundaries continue to be reached in a session with review material and no page is subscribed to the dashboard
- **THEN** the dashboard SHALL keep serving

#### Scenario: A dashboard that died is restored at the next turn boundary
- **WHEN** a turn boundary is reached in a session with review material and no dashboard is serving the root
- **THEN** the system SHALL start one

### Requirement: A hook-started dashboard exits when nothing needs it
A dashboard started by a hook SHALL exit when no page has been subscribed to it and no turn boundary has ensured it for thirty minutes. A dashboard started by hand SHALL NOT exit on idleness.

Both conditions are required. Exiting on an unsubscribed page alone would shut the dashboard down while a session is still working towards the verdict it exists to collect, leaving the reviewer a dead URL when they return. A dashboard started by hand was asked for by someone standing in front of it; a hook-started one was asked for by nobody, which is what earns it a deadline.

#### Scenario: An abandoned dashboard exits
- **WHEN** a hook-started dashboard has had no subscribed page and no ensure for the idle window
- **THEN** it SHALL exit

#### Scenario: An open page holds the dashboard open
- **WHEN** a page is subscribed to a hook-started dashboard for longer than the idle window
- **THEN** the dashboard SHALL keep serving

#### Scenario: A hand-started dashboard is never idled out
- **WHEN** a dashboard started without the idle-exit option has had no subscribed page and no ensure for longer than the idle window
- **THEN** it SHALL keep serving

### Requirement: Registration means a session has something to review
The system SHALL register a session — write its directive record — at a turn boundary, and only when that session has review material: a scratch note, or a change its exploration was promoted to. It SHALL use the same predicate that decides whether to ensure a dashboard.

Every turn boundary in the project runs the turn-end hook, so registering unconditionally makes a directive record mean only that some turn ended in this project. The dashboard index is the visible cost: it lists sessions that never explored, alongside the ones the reviewer is looking for. One predicate should decide registering, ensuring, and listing.

#### Scenario: A session doing unrelated work is not registered
- **WHEN** a turn boundary is reached in a session with no scratch note and no promoted change
- **THEN** the system SHALL NOT write a directive record for it, and the dashboard SHALL NOT list it

#### Scenario: A session with an exploration is registered
- **WHEN** a turn boundary is reached in a session whose scratch note exists and which has no directive record
- **THEN** the system SHALL write one

#### Scenario: Starting an exploration registers nothing on its own
- **WHEN** an exploration starts, before any turn boundary has been reached
- **THEN** the system SHALL NOT write a directive record and SHALL NOT start a dashboard

### Requirement: A browser opens once per session, when its page first has something on it
The system SHALL open a browser at that session's page at the turn boundary on which the session is first registered, and SHALL NOT open one for that session again. It SHALL NOT open one when an exploration starts.

The moment an exploration starts is the wrong moment. The note does not exist yet — readying its location is all that has happened — so the session page renders no artifact at all, and the artifact is not refetched when the page live-updates. A tab opened then shows an empty page that stays empty until the reviewer reloads it by hand.

The first turn boundary with review material is the first moment the page has the note on it, and it is one turn after the reviewer typed the explore command, so it is a page they asked for rather than one that ambushed them. Tying it to first registration rather than to whether a dashboard was started is what bounds it to once per session: a second exploration in the same session finds its record already there and opens nothing, while a second session against the same root gets its own tab for its own page, which is correct.

#### Scenario: The first turn boundary of an exploration opens its page
- **WHEN** a session with a scratch note reaches a turn boundary and has no directive record yet
- **THEN** the system SHALL attempt to open a browser at that session's page

#### Scenario: Later turn boundaries open nothing
- **WHEN** a session that is already registered reaches a further turn boundary
- **THEN** the system SHALL NOT attempt to open a browser

#### Scenario: Re-entering explore mode opens no second tab
- **WHEN** an exploration starts again in a session that is already registered
- **THEN** the system SHALL NOT attempt to open a browser

#### Scenario: A second session against the same root opens its own page
- **WHEN** a second session with its own scratch note first registers while a dashboard is already serving that root
- **THEN** the system SHALL reuse the dashboard and SHALL attempt to open a browser at the second session's page

#### Scenario: A session with nothing to review opens nothing
- **WHEN** a turn boundary is reached in a session with no scratch note and no promoted change
- **THEN** the system SHALL NOT attempt to open a browser
