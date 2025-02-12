# Config search errors
failed-cache-dir = Failed to determine system cache directory
invalid-cache-dir = Cache directory { $dir } does not exist
invalid-system-time = Invalid system time: { $err }
failed-cache-file = Unable to interact with cache file: { $err }
failed-download = Failed to download cache file: { $err }
failed-json = Failed to serialize JSON data: { $err }
required-os = An OS must be specified before searching for releases, editions, or architectures
required-release = A release is required before searching for editions
required-edition = An edition is required before selecting a config
invalid-os = No OS matching { $os } was found
invalid-release = No release { $rel } found for { $os }
invalid-edition = No edition { $edition } found
invalid-arch = Architecture { $arch } not found including other parameters
no-editions = No editions are available for the specified release

# Download errors
unsupported-source = A source does not currently exist for { $os }
invalid-vm-name = Invalid VM name { $vm_name }
config-file-error = Unable to write to config file: { $err }
config-data-error = Unable to serialize config data: { $err }
download-error = File { $file } was not successfully downloaded
invalid-checksum = Invalid checksum: { $cs }
failed-validation = Checksums did not match. Expected { $expected }, got { $actual }
dir-exists = VM Directory { $dir } already exists
