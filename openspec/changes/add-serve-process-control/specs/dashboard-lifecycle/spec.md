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

### Requirement: Assigned ports are listed beside running dashboards and never mistaken for them
The list command SHALL also report ports assigned to a project root with no dashboard serving them, distinguished from the running ones, and SHALL determine that a dashboard is running only from a probe of that port. It SHALL distinguish a dashboard serving on a port other than its root's assignment.

Every project root owns a port whether or not anything is serving it, so "where would this checkout appear" is a question the command can now answer, and an operator scanning the table should not have to know that a project is missing because it is idle rather than because it is unassigned. Reading a running state out of the recorded assignments instead of the network is the failure the whole discovery design exists to prevent, and this command must not be the place it re-enters.

#### Scenario: An assigned project with nothing running is shown as such
- **WHEN** the list command runs and a project root has a port assigned with no dashboard serving it
- **THEN** the output SHALL include that root and port, marked as not running

#### Scenario: A dashboard away from its assignment is distinguishable
- **WHEN** the list command runs and a dashboard is serving a root on a port other than that root's assignment
- **THEN** the output SHALL show the port it is actually serving on and SHALL distinguish it from a dashboard on its assigned port

#### Scenario: A missing record of assignments does not affect the running dashboards
- **WHEN** the list command runs with no recorded assignments available
- **THEN** the system SHALL still report every running dashboard found by probing

### Requirement: A port assignment can be dropped by naming its project
The system SHALL provide a command that drops a project root's port assignment, SHALL require the root to be named, and SHALL refuse to drop an assignment while a dashboard is serving that root.

The port range is finite, and assigning a port to a newly seen root fails rather than colliding when every port belongs to a root that still exists. That failure names this command, so without it the error describes a dead end rather than a way out. Dropping an assignment under a running dashboard would hand that port to another project while a server still occupies it, leaving the forgotten root to discover its own dashboard on a port that now belongs to someone else.

#### Scenario: Forgetting a root frees its port
- **WHEN** the forget command names a root with an assignment and no dashboard serving it
- **THEN** the system SHALL drop that assignment, and the port SHALL become available to a newly seen root

#### Scenario: Forgetting a served root is refused
- **WHEN** the forget command names a root whose dashboard is currently serving
- **THEN** the system SHALL drop nothing and SHALL report that the dashboard must be stopped first

#### Scenario: Forgetting an unassigned root is reported
- **WHEN** the forget command names a root with no assignment
- **THEN** the system SHALL report that there was nothing to forget rather than reporting success

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
