## ADDED Requirements

### Requirement: Running dashboards can be enumerated
The system SHALL provide a command that probes the whole dashboard port range and reports every dashboard found, with the port it is bound to, the canonical project root it serves, and its process id. The command SHALL NOT require a project root, and SHALL report the range it searched.

Dashboards are global to the machine and started detached by hooks, so an operator can have several running for roots they did not choose to remember. Enumerating them is the prerequisite for stopping the right one; naming the searched range is what lets an operator who does not find their server understand why rather than concluding the tool is broken.

#### Scenario: Every running dashboard is listed
- **WHEN** the list command runs and dashboards are serving two different project roots
- **THEN** the output SHALL include both, each with its port, canonical root, and process id

#### Scenario: Listing works outside a project
- **WHEN** the list command runs from a directory that is not inside any OpenSpec project
- **THEN** the system SHALL still report every dashboard found

#### Scenario: No dashboards is not an error
- **WHEN** the list command runs and no dashboard answers on any port in the range
- **THEN** the system SHALL report that none are running and SHALL exit zero

#### Scenario: A port outside the range is not found
- **WHEN** a dashboard is serving on a port outside the searched range
- **THEN** the system SHALL NOT list it, and the reported range SHALL make the omission explicable

### Requirement: Stopping a dashboard requires naming which one
The system SHALL provide a command that stops a running dashboard, and SHALL require a target identifying it — a project root, a port, or a process id — or an explicit option meaning all of them. It SHALL NOT stop anything when invoked with no target.

Dashboards are machine-global while the operator's mental model is per-project, so the natural reading of an untargeted stop is "the one for this project" rather than "every one on this machine". A command whose destructive behaviour is the default is one an operator runs by accident while another checkout's review is open.

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

### Requirement: A stop reports what is actually gone
The system SHALL confirm the outcome of a stop by probing again after signalling, and SHALL report the dashboard as stopped only when it no longer answers. It SHALL NOT report success on the basis of having sent a signal.

The process id is read from a socket and is a snapshot: by the time the signal is sent the process may have exited on its own idle deadline, and its id may have been reused by something unrelated. Re-probing is what distinguishes "this dashboard is gone" from "a signal was delivered somewhere".

#### Scenario: A dashboard that stops is confirmed by its port going quiet
- **WHEN** a dashboard is signalled and its port stops answering
- **THEN** the system SHALL report it stopped

#### Scenario: A dashboard that survives the signal is reported as still running
- **WHEN** a dashboard is signalled and its port still answers for the same root afterwards
- **THEN** the system SHALL report it as still running rather than as stopped

#### Scenario: A dashboard that had already exited is not reported as killed
- **WHEN** the target has already exited before it is signalled
- **THEN** the system SHALL report that it was not running rather than that it was stopped
