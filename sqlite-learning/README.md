# sqlite-learning

I believe that in order to create the `frzr` tool, I will want to essentially glue together three crates:
 * `sqlite`, for storing the file metadata
 * `walkdir`, for actually traversing the file system
 * `crypto-hashes`, for generating checksums of file data

 This subdirectory contains the learning code/playground that will enable me to understand how to use the sqlite crate
