# frzr
A bitrot detection tool

**Note: frzr is alpha; it is not complete nor correct. Read all the code before you try it!**

## What does frzr do?
`frzr` is intended to replace and enhance the process of creating, storing, and checking checksums of
files over time. In the past, you may have done something like:
```bash
# Create MD5SUMS.md5, containing md5 hashes of all files under . 
time find . -type f -print0 |xargs -0 -n 1000 -P 6 md5sum |tee ~/MD5SUMS.md5

# Later, check whether any file has changed:
md5sum -c ~/MD5SUMS.md5 |grep -v -e 'OK$'
```

This pattern is useful for files that you want to keep "on ice"; whose contents you do not expect to change. When the
files _do_ intentionally change, though, you have to look at each file and make sure the changes were intentional and
then generate the MD5SUMS.md5 file again. Additionally, you may say to yourself "Hey, this MD5SUMS.md5 file is
basically" a database. It sure would be nice to get some stats out of it.". This is where frzr comes in.

The aim of frzr is to make the database of checksums of your important files easy to create, manage, and use.
Eventually, I want frzr to be able to:
```bash
# Iterate over all files and directories under $PWD and create checksums:
frzr check

# Print a summary of last `frzr check` run, reading from the checksum database only:
frzr report

# Accept any changes from the last run:
frzr resolve

# Show all duplicate files
frzr report --dupes
```