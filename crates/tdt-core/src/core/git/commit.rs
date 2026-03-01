//! Commit creation, tags, and log/history using gix

use gix::bstr::ByteSlice;
use gix::object::tree::EntryKind;

use super::{format_gix_time, gix_err, CommitLogEntry, Git, GitError};

impl Git {
    /// Commit staged changes, returning the new commit hash
    pub fn commit(&self, message: &str) -> Result<String, GitError> {
        let repo = self.open_repo()?;

        // Build a tree from the current index
        let index = repo.index_or_empty().map_err(gix_err)?;
        let tree_id = self.build_tree_from_index(&repo, &index)?;

        // Determine parents: if HEAD exists, it's the parent
        let parents: Vec<gix::ObjectId> = match repo.head_id() {
            Ok(head) => vec![head.detach()],
            Err(_) => vec![], // Initial commit — no parents
        };

        // Create the commit, updating HEAD
        let commit_id = repo
            .commit("HEAD", message, tree_id, parents)
            .map_err(gix_err)?;

        Ok(commit_id.to_hex().to_string())
    }

    /// Create an annotated tag
    pub fn create_tag(&self, name: &str, message: Option<&str>) -> Result<(), GitError> {
        let repo = self.open_repo()?;
        let head_id = repo.head_id().map_err(gix_err)?;

        if let Some(msg) = message {
            // Annotated tag
            let tagger = repo.committer().ok_or_else(|| GitError::CommandFailed {
                message: "Committer identity not configured".to_string(),
            }).and_then(|r| r.map_err(gix_err))?;

            repo.tag(
                name,
                head_id.detach(),
                gix::objs::Kind::Commit,
                Some(tagger),
                msg,
                gix::refs::transaction::PreviousValue::MustNotExist,
            )
            .map_err(gix_err)?;
        } else {
            // Lightweight tag — just a reference
            repo.reference(
                format!("refs/tags/{}", name),
                head_id.detach(),
                gix::refs::transaction::PreviousValue::MustNotExist,
                "tag: lightweight",
            )
            .map_err(gix_err)?;
        }
        Ok(())
    }

    /// Get recent commits from HEAD
    pub fn recent_commits(&self, limit: u32) -> Result<Vec<CommitLogEntry>, GitError> {
        let repo = self.open_repo()?;
        let head_id = repo.head_id().map_err(gix_err)?;

        let walk = repo
            .rev_walk([head_id.detach()])
            .sorting(gix::revision::walk::Sorting::ByCommitTime(
                gix::traverse::commit::simple::CommitTimeOrder::NewestFirst,
            ))
            .all()
            .map_err(gix_err)?;

        let mut entries = Vec::new();
        for info in walk {
            if entries.len() >= limit as usize {
                break;
            }
            let info = info.map_err(gix_err)?;
            let commit = info.object().map_err(gix_err)?;
            let decoded = commit.decode().map_err(gix_err)?;

            let hash = info.id.to_hex().to_string();
            let short_hash = hash[..7].to_string();
            let message = decoded
                .message
                .to_str_lossy()
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            let author = decoded.author.name.to_str_lossy().to_string();
            let author_email = decoded.author.email.to_str_lossy().to_string();
            let date = decoded
                .author
                .time()
                .ok()
                .map(format_gix_time)
                .unwrap_or_default();

            // Check for GPG signature in extra headers
            let is_signed = decoded
                .extra_headers
                .iter()
                .any(|(key, _)| *key == "gpgsig");

            entries.push(CommitLogEntry {
                hash,
                short_hash,
                message,
                author,
                author_email,
                date,
                is_signed,
            });
        }
        Ok(entries)
    }

    /// Build a tree object from the current index entries
    fn build_tree_from_index(
        &self,
        repo: &gix::Repository,
        index: &gix::index::File,
    ) -> Result<gix::ObjectId, GitError> {
        let empty_tree = repo.empty_tree();
        let mut editor = empty_tree.edit().map_err(gix_err)?;

        for entry in index.entries() {
            let path = entry.path_in(index.path_backing());
            let path_str = path.to_str_lossy();

            let kind = match entry.mode {
                gix::index::entry::Mode::FILE => EntryKind::Blob,
                gix::index::entry::Mode::FILE_EXECUTABLE => EntryKind::BlobExecutable,
                gix::index::entry::Mode::SYMLINK => EntryKind::Link,
                gix::index::entry::Mode::COMMIT => EntryKind::Commit,
                _ => EntryKind::Blob,
            };

            editor
                .upsert(path_str.as_ref(), kind, entry.id)
                .map_err(gix_err)?;
        }

        let tree_id = editor.write().map_err(gix_err)?;
        Ok(tree_id.detach())
    }
}
