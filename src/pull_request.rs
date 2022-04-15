use anyhow::{bail, Result};

static EXPECTED_PARTS_OF_RESOURCE: usize = 4;

#[derive(Debug, PartialEq, Clone)]
pub struct PullRequest {
    pub url: Option<String>,
    pub owner: String,
    pub repo: String,
    pub request: u64,
}

impl Default for PullRequest {
    fn default() -> Self {
        Self {
            url: None,
            owner: String::new(),
            repo: String::new(),
            request: 0,
        }
    }
}

impl PullRequest {
    pub fn new(resource: String) -> Result<Self> {
        if !Self::valid(&resource) {
            bail!("Invalid url for pull request");
        }

        let resource_parts: Vec<&str> = resource.split("/").collect();
        let length = resource_parts.len();

        Ok(Self {
            url: Some(resource.clone()),
            owner: resource_parts[length - 4].to_string(),
            repo: resource_parts[length - 3].to_string(),
            request: resource_parts[length - 1].parse::<u64>()?,
        })
    }

    pub fn valid(resource: &str) -> bool {
        let normalized_resource = resource
            .replace("https://github.com/", "")
            .replace("//", "/")
            .replace("github.com/", "");

        if !normalized_resource.contains("pull") {
            return false;
        }

        if normalized_resource.matches("/").count() < (EXPECTED_PARTS_OF_RESOURCE - 1) {
            return false;
        }

        true
    }

    pub fn same(&self, other: &PullRequest) -> bool {
        self.url == other.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_with_full_url() {
        assert_eq!(
            PullRequest::valid("https://github.com/Liberatys/rusty-merge/pull/400"),
            true
        );

        assert_eq!(
            PullRequest::valid("github.com/Liberatys/rusty-merge/pull/400"),
            true
        );
    }

    #[test]
    fn test_valid_with_relative_pull_request() {
        assert_eq!(PullRequest::valid("Liberatys/rusty-merge/pull/400"), true);
    }

    #[test]
    fn test_valid_with_missing_pull() {
        assert_eq!(PullRequest::valid("Liberatys/rusty-merge/400"), false);

        assert_eq!(
            PullRequest::valid("https://github.com/Liberatys/rusty-merge/400"),
            false
        );
    }

    #[test]
    fn test_valid_with_missing_repo() {
        assert_eq!(PullRequest::valid("Liberatys/pull/400"), false);

        assert_eq!(
            PullRequest::valid("https://github.com/Liberatys/pull/400"),
            false
        );
    }

    #[test]
    fn test_valid_with_duplicate_slashes() {
        assert_eq!(PullRequest::valid("Liberatys/rusty-merge//pull/400"), true);

        assert_eq!(
            PullRequest::valid("https://github.com/Liberatys/rusty-merge//pull/400"),
            true
        );
    }

    #[test]
    fn test_same() {
        assert_eq!(
            PullRequest {
                url: Some("://url.com".to_string()),
                ..PullRequest::default()
            }
            .same(&PullRequest {
                url: Some("https://url.com".to_string()),
                ..PullRequest::default()
            }),
            false
        );

        assert_eq!(
            PullRequest {
                url: Some("https://url.com".to_string()),
                ..PullRequest::default()
            }
            .same(&PullRequest {
                url: Some("https://url.com".to_string()),
                ..PullRequest::default()
            }),
            true
        );
    }
}
