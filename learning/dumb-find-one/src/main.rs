// The goal of this tiny, hopefully-only-one-file project is to:
//  1. Traverse the file system
//  2. Print all filenames we come across
// While learning along the way. Should learn a little about what edge cases we find, like maybe
// figure out how not to traverse symlinks and stuff

use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;


fn main() {
    // We'll do a depth-first-search of the file system, so we should:
    // Keep a list of our current path components, starting with '.'
    // Keep a pointer to our current dir
    // Keep a list of "we're all the way done with this dir" dirnames
    // Read all dirnames in the current dir into memory
    // Pick an order to go in (sort all dirnames first, or just let it be whatever the data
    // structure gives us.
    // Go the first dir in the list
    // Do it all over again
    // Base case: we're in a dir with no child dirs or a dir in which we have already been to all
    // child dirs of. Then, we print all non-dir files and mark the dir as complete
    let file_entry_vec : Vec<DirEntry> = Vec::new();
    let path_buf : PathBuf = PathBuf::from(".");
    println!("PathBuf:\t{:?}", path_buf);
    dir_walk_recurser(path_buf, file_entry_vec);
    println!{"Reached end of main"};
}

fn dir_walk_recurser(path_string: PathBuf, mut visited_dirs: Vec<DirEntry>) -> Vec<DirEntry> {
    let dir_iter = match fs::read_dir(path_string) {
        Ok(rd) => rd,
        Err(e) => {
            println!("An error occurred: {}", e);
            return visited_dirs;
        }
    };
    // println!("{:?}", dir_iter);
    for entry in dir_iter {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                println!("An error occurred: {}", e);
                return visited_dirs;
            }
        };
        // println!("{:?}", entry);
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                println!("An error occurred: {}", e);
                return visited_dirs;
            }
        };
        let path_name = entry.path();
        if file_type.is_dir() {
            println!("Dir:\t{:?}", path_name);
            // add this to the list and recurse
            visited_dirs.push(entry);

            visited_dirs = dir_walk_recurser(path_name, visited_dirs);
        } else {
            println!("File:\t{:?}", path_name);
            // print this filename
        }
    }
    visited_dirs
}
