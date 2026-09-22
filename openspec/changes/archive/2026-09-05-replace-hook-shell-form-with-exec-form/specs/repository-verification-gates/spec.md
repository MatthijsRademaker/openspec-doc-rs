## MODIFIED Requirements

### Requirement: The platforms the gates are run on are stated, and an uncovered platform is a stated omission

The repository SHALL record which platforms its automated gate runs cover, SHALL cover Linux, macOS, and Windows, and SHALL run the full test suite on each of them with no test excluded by conditional compilation on the ground of the platform's own facilities. A platform the repository distributes a binary for and does not run its gates on SHALL be recorded as an omission, with the reason, and SHALL NOT be represented as covered.

The gap this closes is specific. The workspace's test suite is very nearly platform-agnostic, so an automated run on an unprepared platform passes while silently omitting exactly the tests covering the code that platform breaks. A passing run that omitted the relevant tests is a stronger false claim than no run at all.

Windows was that case and is no longer. The three tests previously compiled out there covered the setup diagnostics and the hook-start path — the two places whose Windows defects were real — so a Windows run with them excluded would have reported success over the only code that was broken. Where a test is excluded on a platform for reasons of the *test harness* rather than the product, the harness is what must change: a symbolic link and a hardcoded path separator are properties of how a fixture was built, not statements about what the product supports.

#### Scenario: The suite runs whole on every covered platform

- **WHEN** the gates run on Linux, macOS, or Windows
- **THEN** every test in the workspace suite SHALL be compiled and run, and no test SHALL be excluded because of the platform it is running on

#### Scenario: A platform whose tests are partly compiled out is not claimed as covered

- **WHEN** a platform would run the suite with some tests excluded by conditional compilation
- **THEN** the repository SHALL record it as not covered, naming the excluded tests, rather than adding a run that reports success

#### Scenario: A fixture that only one platform can build is rebuilt rather than skipped

- **WHEN** a test's fixture relies on a facility one platform lacks, such as a symbolic link or a platform-specific path separator
- **THEN** the fixture SHALL be constructed by portable means, and the test SHALL NOT be excluded on that platform

#### Scenario: Coverage and support move together

- **WHEN** a platform is added to the set the repository distributes a binary for
- **THEN** the automated gate run SHALL cover that platform in the same change, or the change SHALL record why it does not
