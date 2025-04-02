invalid-architecture = Invalid architecture: { $architecture }
list-specified-os = An operating system must not be specified for list operations
docker-command-failed = Failed to run docker command: { $command }

unspecified-os =
    You must specify an operating system
     - Supported Operating Systems
    { $operating_systems }

releases = Releases
editions = Editions

# Releases variable will be formatted within code, since it can be dynamic depending on if all releases have the same editions
# This is why the plurals for releases & editions are included as separate localized values
unspecified-release =
    You must specify a release
    { $releases }

unspecified-edition =
    You must specify an edition
     - Editions: { $editions }
