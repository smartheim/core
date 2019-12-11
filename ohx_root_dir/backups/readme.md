# Backups

This directory contains compressed (.tar.gz) backups.
Depending on the user selection a backup can be on of the following types:

* "full": Includes the entire partition with external software containers.
  Restoring this type of backup restores everything, but is also very big in file size.
* "reinstall": Includes ohx, ohx addons and a list of external software containers and their settings.
  After this type of backup is restored, the external software containers will begin to download.
* "ohx_only": This only backups the "ohx" directory (excluding the backup directory).
  This includes ohx settings, addon settings, scripts, rules, devices, profiles, users.
  Addons will be reinstalled after restoring this type of backup.
  The file size of this type is minimal and allows for "a backup a day" scenarios.