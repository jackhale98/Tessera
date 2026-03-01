//! Repository queries using gix — branches, tags, HEAD, config, status

use gix::bstr::ByteSlice;

use super::{format_gix_time, gix_err, glob_match, BranchInfoEntry, Git, GitError, TagInfoEntry};

impl Git {
    /// Check if we're in a git repository
    pub fn is_repo(&self) -> bool {
        gix::open(&self.repo_root).is_ok()
    }

    /// Get current branch name
    pub fn current_branch(&self) -> Result<String, GitError> {
        let repo = self.open_repo()?;
        match repo.head_name().map_err(gix_err)? {
            Some(name) => Ok(name.shorten().to_string()),
            None => {
                // Detached HEAD — return the short hash
                let id = repo.head_id().map_err(gix_err)?;
                Ok(format!("{:.7}", id))
            }
        }
    }

    /// Check if on main or master branch
    pub fn is_main_branch(&self) -> bool {
        self.current_branch()
            .map(|b| b == "main" || b == "master")
            .unwrap_or(false)
    }

    /// Check if working directory is clean (no uncommitted changes)
    pub fn is_clean(&self) -> bool {
        let repo = match self.open_repo() {
            Ok(r) => r,
            Err(_) => return false,
        };
        match repo.is_dirty() {
            Ok(dirty) => !dirty,
            Err(_) => {
                // Fallback: check status iterator
                self.uncommitted_files()
                    .map(|f| f.is_empty())
                    .unwrap_or(false)
            }
        }
    }

    /// Get list of uncommitted changes (status --porcelain style)
    pub fn uncommitted_files(&self) -> Result<Vec<String>, GitError> {
        let repo = self.open_repo()?;
        let status = repo.status(gix::progress::Discard).map_err(gix_err)?;
        let iter = status
            .into_iter(Vec::<gix::bstr::BString>::new())
            .map_err(gix_err)?;

        let mut files = Vec::new();
        for item in iter {
            let item = item.map_err(gix_err)?;
            let path = item.location().to_str_lossy().to_string();
            files.push(path);
        }
        Ok(files)
    }

    /// Get git user.name from config
    pub fn user_name(&self) -> Result<String, GitError> {
        let repo = self.open_repo()?;
        let snap = repo.config_snapshot();
        snap.string("user.name")
            .map(|v| v.to_str_lossy().to_string())
            .ok_or_else(|| GitError::CommandFailed {
                message: "git user.name not configured".to_string(),
            })
    }

    /// Get git user.email from config
    pub fn user_email(&self) -> Result<String, GitError> {
        let repo = self.open_repo()?;
        let snap = repo.config_snapshot();
        snap.string("user.email")
            .map(|v| v.to_str_lossy().to_string())
            .ok_or_else(|| GitError::CommandFailed {
                message: "git user.email not configured".to_string(),
            })
    }

    /// Get the GPG signing key ID
    pub fn signing_key(&self) -> Option<String> {
        let repo = self.open_repo().ok()?;
        let snap = repo.config_snapshot();
        snap.string("user.signingkey")
            .map(|v| v.to_str_lossy().to_string())
    }

    /// Check if GPG signing is configured
    pub fn signing_configured(&self) -> bool {
        self.signing_key().is_some()
    }

    /// Check if commit.gpgsign is enabled
    pub fn commit_gpgsign_enabled(&self) -> bool {
        self.open_repo()
            .ok()
            .and_then(|repo| repo.config_snapshot().boolean("commit.gpgsign"))
            .unwrap_or(false)
    }

    /// Check if tag.gpgSign is enabled
    pub fn tag_gpgsign_enabled(&self) -> bool {
        self.open_repo()
            .ok()
            .and_then(|repo| repo.config_snapshot().boolean("tag.gpgSign"))
            .unwrap_or(false)
    }

    /// Check if a local branch exists
    pub fn branch_exists(&self, name: &str) -> bool {
        let repo = match self.open_repo() {
            Ok(r) => r,
            Err(_) => return false,
        };
        repo.try_find_reference(&format!("refs/heads/{}", name))
            .ok()
            .flatten()
            .is_some()
    }

    /// Check if a remote branch exists
    pub fn remote_branch_exists(&self, remote: &str, branch: &str) -> bool {
        let repo = match self.open_repo() {
            Ok(r) => r,
            Err(_) => return false,
        };
        repo.try_find_reference(&format!("refs/remotes/{}/{}", remote, branch))
            .ok()
            .flatten()
            .is_some()
    }

    /// Create a new branch (without checking out)
    pub fn create_branch(&self, name: &str) -> Result<(), GitError> {
        if self.branch_exists(name) {
            return Err(GitError::BranchExists {
                branch: name.to_string(),
            });
        }
        let repo = self.open_repo()?;
        let head_id = repo.head_id().map_err(gix_err)?;
        repo.reference(
            format!("refs/heads/{}", name),
            head_id.detach(),
            gix::refs::transaction::PreviousValue::MustNotExist,
            gix::bstr::BString::from("branch: Created from HEAD"),
        )
        .map_err(gix_err)?;
        Ok(())
    }

    /// List all local branches
    pub fn list_local_branches(&self) -> Result<Vec<BranchInfoEntry>, GitError> {
        let repo = self.open_repo()?;
        let refs = repo.references().map_err(gix_err)?;
        let mut branches = Vec::new();

        for reference in refs.local_branches().map_err(gix_err)? {
            let reference = reference.map_err(gix_err)?;
            let name = reference.name().shorten().to_string();
            let commit_id = reference.id().detach();
            let hex = commit_id.to_hex().to_string();
            let message = match repo.find_commit(commit_id) {
                Ok(commit) => commit.message_raw_sloppy().to_str_lossy().to_string(),
                Err(_) => String::new(),
            };
            branches.push(BranchInfoEntry {
                name,
                commit: hex,
                message: message.lines().next().unwrap_or("").to_string(),
            });
        }
        Ok(branches)
    }

    /// List remote branches
    pub fn list_remote_branches(&self) -> Result<Vec<BranchInfoEntry>, GitError> {
        let repo = self.open_repo()?;
        let refs = repo.references().map_err(gix_err)?;
        let mut branches = Vec::new();

        for reference in refs.remote_branches().map_err(gix_err)? {
            let reference = reference.map_err(gix_err)?;
            let name = reference.name().shorten().to_string();
            // Skip HEAD pointers
            if name.contains("HEAD") {
                continue;
            }
            let commit_id = reference.id().detach();
            let hex = commit_id.to_hex().to_string();
            let message = match repo.find_commit(commit_id) {
                Ok(commit) => commit.message_raw_sloppy().to_str_lossy().to_string(),
                Err(_) => String::new(),
            };
            branches.push(BranchInfoEntry {
                name,
                commit: hex,
                message: message.lines().next().unwrap_or("").to_string(),
            });
        }
        Ok(branches)
    }

    /// Get the current HEAD commit SHA (full 40-char)
    pub fn head_sha(&self) -> Result<String, GitError> {
        let repo = self.open_repo()?;
        let id = repo.head_id().map_err(gix_err)?;
        Ok(id.to_hex().to_string())
    }

    /// Get the short commit SHA (first 7 characters)
    pub fn head_sha_short(&self) -> Result<String, GitError> {
        let sha = self.head_sha()?;
        Ok(sha[..7].to_string())
    }

    /// Check if a tag exists
    pub fn tag_exists(&self, name: &str) -> bool {
        let repo = match self.open_repo() {
            Ok(r) => r,
            Err(_) => return false,
        };
        repo.try_find_reference(&format!("refs/tags/{}", name))
            .ok()
            .flatten()
            .is_some()
    }

    /// List tags, optionally filtered by a glob pattern
    pub fn list_tags(&self, pattern: Option<&str>) -> Result<Vec<String>, GitError> {
        let repo = self.open_repo()?;
        let refs = repo.references().map_err(gix_err)?;
        let mut tags = Vec::new();

        for reference in refs.tags().map_err(gix_err)? {
            let reference = reference.map_err(gix_err)?;
            let name = reference.name().shorten().to_string();
            if let Some(pat) = pattern {
                if !glob_match(pat, &name) {
                    continue;
                }
            }
            tags.push(name);
        }
        tags.sort();
        Ok(tags)
    }

    /// Get tag info (tagger, date, message, commit)
    pub fn tag_info(&self, name: &str) -> Result<TagInfoEntry, GitError> {
        let repo = self.open_repo()?;
        let reference = repo
            .find_reference(&format!("refs/tags/{}", name))
            .map_err(gix_err)?;

        let obj = reference.id().object().map_err(gix_err)?;

        if obj.kind == gix::object::Kind::Tag {
            let tag = obj.into_tag();
            let decoded = tag.decode().map_err(gix_err)?;
            let tagger_str = decoded
                .tagger
                .as_ref()
                .map(|t| format!("{} <{}>", t.name, t.email));
            let date_str = decoded
                .tagger
                .as_ref()
                .and_then(|t| t.time().ok().map(format_gix_time));
            Ok(TagInfoEntry {
                tagger: tagger_str,
                date: date_str,
                message: Some(decoded.message.to_str_lossy().to_string()),
                commit: Some(decoded.target.to_str_lossy().to_string()),
            })
        } else {
            // Lightweight tag pointing directly to a commit
            Ok(TagInfoEntry {
                tagger: None,
                date: None,
                message: None,
                commit: Some(obj.id.to_hex().to_string()),
            })
        }
    }

    /// Get the default remote name (usually "origin")
    pub fn default_remote(&self) -> Result<String, GitError> {
        let repo = self.open_repo()?;
        let snap = repo.config_snapshot();
        // Try to get remote.origin.url — if it exists, "origin" is the default
        if snap.string("remote.origin.url").is_some() {
            return Ok("origin".to_string());
        }
        // Otherwise find first remote
        match repo.remote_names().first() {
            Some(name) => Ok(name.to_string()),
            None => Ok("origin".to_string()),
        }
    }

    /// Get the URL of a remote
    pub fn remote_url(&self, remote: &str) -> Result<String, GitError> {
        let repo = self.open_repo()?;
        let snap = repo.config_snapshot();
        let key = format!("remote.{}.url", remote);
        snap.string(&key)
            .map(|v| v.to_str_lossy().to_string())
            .ok_or_else(|| GitError::CommandFailed {
                message: format!("Remote '{}' not found", remote),
            })
    }

    /// Get the base branch (main or master)
    pub fn base_branch(&self) -> String {
        if self.branch_exists("main") {
            return "main".to_string();
        }
        if self.branch_exists("master") {
            return "master".to_string();
        }
        "main".to_string()
    }

    /// Resolve a reference to its commit SHA
    pub fn rev_parse(&self, reference: &str) -> Result<String, GitError> {
        let repo = self.open_repo()?;
        let id = repo.rev_parse_single(reference).map_err(gix_err)?;
        Ok(id.to_hex().to_string())
    }
}
