# frzr – a tool that watches your data for unintended changes
*As of March 2020, frzr is still vaporware. This README will be updated as actual progress is made toward making frzr a reality*

## Marketing Content
frzr is a tool that watches your data for unintended changes. It is intended to be one component in your data integrity toolbox, and works best when paired with a robust backup solution. It allows you to detect file corruption caused by any means, and react accordingly in a timely manner. The intended use case is to run it periodically or in daemon mode to watch files that aren't intended to change often and react to any alert by restoring the affected data from a good backup and replacing any failing storage media.

With frzr and a good backup solution, you can cheaply keep large amounts of data with high confidence on cheap disks and sub-par filesystems for long periods of time. It is an ideal tool for the archivist on a budget or anyone who wants to keep family photos safe for generations to come (and has a healthy paranoia around off-prem, closed, or third-party solutions).

The best thing about frzr is that it operates above the filesystem layer. It simulates manually verifying the contents of your precious files the same way you would by opening the file and looking at its contents. Thus, it is like a true end-to-end test for your data. Is you data corrupted? Prove it! By opening your files or by letting frzr do it for you.

Great for folks who don't trust their filesystem, operating system, or third-party cloud storage provider to return the same bytes written.

frzr cannot save you from data corruption, but it can help you recognize if corruption has occurred. Thus, it aims to eliminate silent corruption. The only thing worse than data corruption is silent data corruption.

Do you want to save home movies, pictures, essays, letters, artwork, or any digital artifact for years without having to think about it? And be certain that when you decide to open the file years later, the contents will be intact as you left them? Then frzr is for you.

## Desired Interface
Here's a summary of the "verbs" that we want `frzr` to have:
* `frzr init` – Performs initial one-time setup: choose a place for the master database to live
* `frzr freeze` – Takes a directory name as an argument. Walks the directory tree, records vital statistics on each file, and reports any change in vital statistics if found.
* `frzr report` – Prints summary of last `frzr freeze` run (doesn't walk the directory tree; just reports what's in the DB from the previous run).
* `frzr status` – Prints whether `frzr` is currently running (useful for daemon mode), maybe shows progress also?
* `frzr resolve` – Presents changes with option to accept as new content (could be rolled into `report` via `frzr report --interactive`?)
* `frzr list` – Lists all "freezers", which are directory roots of that have had `freeze` run for them

## Implementation
*Remember, frzr is still vaporware; this is a wishlist*

`frzr` needs to be able to walk the filesystem intelligently (symlinks etc) and read/write to DB in config directory intelligently (multiple processes don't collide or step on each other; probably enforce only one `freeze` operation at a time per DB?)

### Default files/configuration
.local/frzr/ should contain the master configration for `frzr`. The mater database will live here by default, but it can be configured to be elsewhere with `frzr init`

### Database schema
`freezer` – root for for tree
`file` – many per `freezer`. Relative path from freezer root, so moving frozen files is easy. Nested freezers elide contents from parent so only one freezer per given file
`file_vitals` – snapshot of file metadata (checksums, etc) for a single file. Files have many of these, for history

Freezers may contain zero or more files and zero or more Freezers (probably not first-level relationship in DB for Freezer->Freezer, just compute when necessary?)

## Special Features (not MVP)
* Daemon mode – Continuously or on interval, walk tree, checking contents against stored vitals. On change, alert/email or just write new `file_vitals` to be seen later with the appropriate verb.
* Daemon mode tuning – Ability to tune back by CPU perentage or IO throughput to continuously run without negative system impact
* Parallel `freeze` – Ability to tuned `freeze` to run in parallel intelligently on CPU/IO capabilities of machine for fastest possible operation at the expense of system responsiveness
* `--all` option for verbs – Ability for `freeze`/`report`/`resolve` to take `--all` and go for all freezers at once, or `--recursive` if one big root freezer. Optionally parallelize as described above
