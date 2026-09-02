## ADDED Requirements

### Requirement: A dashboard can be asked to stop over the port it serves
The dashboard SHALL expose a route that begins its graceful shutdown, SHALL accept it only as a write request carrying a header a cross-origin form cannot set, SHALL require the caller to name the project root it believes it is stopping and refuse the request when that root is not the one being served, and SHALL answer the request before exiting.

The port is the dashboard's identity in a way a process id is not: a pid may be reused between learning it and acting on it, while a port that stops answering is the evidence directly. Asking the server to stop itself also lets it check what it is, which closes the case where the target port changed hands between the enumeration and the request — a signal cannot ask that question. Requiring a non-simple header keeps a page in the operator's own browser from stopping a dashboard on a guessable local port. Answering first is what lets the caller distinguish a shutdown that started from a request that never arrived.

#### Scenario: A shutdown request stops the dashboard
- **WHEN** the shutdown route is called on a dashboard, naming the root that dashboard serves
- **THEN** the dashboard SHALL answer the request and SHALL then shut down gracefully

#### Scenario: A request naming another root is refused
- **WHEN** the shutdown route is called naming a project root other than the one the dashboard serves
- **THEN** the dashboard SHALL refuse the request and SHALL keep serving

#### Scenario: A request a browser form could have sent is refused
- **WHEN** the shutdown route is called without the required header
- **THEN** the dashboard SHALL refuse the request and SHALL keep serving

#### Scenario: The idle exit still applies
- **WHEN** a dashboard with the shutdown route available reaches its idle condition
- **THEN** it SHALL exit as it did before, and a shutdown arriving alongside that exit SHALL NOT fail

### Requirement: Stopping a dashboard requires naming which one
The system SHALL provide a command that stops a running dashboard, and SHALL require a target identifying it — an explicit option meaning the resolved project, a port, or an explicit option meaning all of them. It SHALL NOT stop anything when invoked with no target, and the option meaning the resolved project SHALL be given explicitly rather than applied as a default.

Dashboards are machine-global while the operator's mental model is per-project, so the natural reading of an untargeted stop is "the one for this project" rather than "every one on this machine". A command whose destructive behaviour is the default is one an operator runs by accident while another checkout's review is open. Making the project target explicit is what keeps "stop this project's dashboard" from being indistinguishable from having named no target at all, since the project is what the system resolves when no root is given.

#### Scenario: A targeted stop ends one dashboard
- **WHEN** the stop command names one of several running dashboards
- **THEN** the system SHALL stop that dashboard and SHALL leave the others serving

#### Scenario: An untargeted stop does nothing
- **WHEN** the stop command runs with no target and no all-option
- **THEN** the system SHALL stop no dashboard and SHALL report that a target is required

#### Scenario: Stopping everything is available but explicit
- **WHEN** the stop command runs with the all-option
- **THEN** the system SHALL stop every dashboard found in the range

#### Scenario: A target matching nothing is reported
- **WHEN** the stop command names a target that no running dashboard matches
- **THEN** the system SHALL report that nothing matched rather than reporting success

#### Scenario: Stopping a dashboard leaves its project's port assignment in place
- **WHEN** a project's dashboard is stopped and a dashboard for that project is later ensured again
- **THEN** it SHALL be served on the same assigned port

### Requirement: A stop reports what is actually gone
The system SHALL resolve its target by probing immediately before asking a dashboard to stop, SHALL confirm the outcome by probing again afterwards, and SHALL report the dashboard as stopped only when it no longer answers. It SHALL NOT report success on the basis of having sent a request.

A resolved target is a snapshot: by the time the request is sent the dashboard may have exited on its own idle deadline, been stopped by someone else, or had its port taken by another project's dashboard falling forward onto it. Re-probing is what distinguishes "this dashboard is gone" from "a request was delivered somewhere".

#### Scenario: A dashboard that stops is confirmed by its port going quiet
- **WHEN** a dashboard is asked to stop and its port stops answering
- **THEN** the system SHALL report it stopped

#### Scenario: A dashboard that survives the request is reported as still running
- **WHEN** a dashboard is asked to stop and its port still answers for the same root afterwards
- **THEN** the system SHALL report it as still running rather than as stopped

#### Scenario: A dashboard that had already exited is not reported as killed
- **WHEN** the target has already exited before it is asked to stop
- **THEN** the system SHALL report that it was not running rather than that it was stopped
