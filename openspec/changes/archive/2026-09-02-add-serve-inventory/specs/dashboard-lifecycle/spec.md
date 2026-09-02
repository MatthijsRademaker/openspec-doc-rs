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

