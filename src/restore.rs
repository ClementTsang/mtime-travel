use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use filetime::FileTime;

use crate::{file_entry::FileEntry, hash::hash_file};

/// Restore the mtimes.
pub(crate) fn restore_mtimes(
    mtime_file_path: PathBuf,
    target_dir: PathBuf,
    verbose: bool,
    ignore_hash: bool,
) -> Result<()> {
    let data: BTreeMap<String, FileEntry> = {
        let mtime_file = File::open(&mtime_file_path).context(format!(
            "restore file at `{}` can't be opened",
            mtime_file_path.to_string_lossy()
        ))?;
        serde_json::from_reader(mtime_file)?
    };

    println!(
        "Restoring from `{}` to `{}`",
        mtime_file_path.to_string_lossy(),
        target_dir.to_string_lossy()
    );

    if verbose && ignore_hash {
        println!("Note that hashes are being ignored!");
    }

    let mut num_restored = 0;

    for (file, entry) in data {
        let path = Path::new(&file);

        if !path.exists() {
            continue;
        }

        if !ignore_hash {
            let Ok(hash) = hash_file(path) else {
                if verbose {
                    eprintln!(
                        "Unable to get hash for `{}`, skipping.",
                        path.to_string_lossy(),
                    );
                }

                continue;
            };

            if hash != entry.hash {
                println!(
                    "Skipping restore for {} due to hash mismatch (`{}` vs `{}`).",
                    file, hash, entry.hash
                );

                continue;
            }
        }

        let mtime = FileTime::from_unix_time(entry.mtime, entry.mtime_nano);
        if let Err(err) = filetime::set_file_mtime(path, mtime) {
            if verbose {
                eprintln!("Unable to set mtime for `{file}` due to {err:?}.");
            }

            continue;
        }

        num_restored += 1;
    }

    println!("Restore complete, restored {num_restored} mtimes.");

    Ok(())
}
