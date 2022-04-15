use octocrab::models::pulls::MergeableState;
use octocrab::models::pulls::PullRequest;
use octocrab::models::IssueState;

pub fn should_pop(pull_request: &PullRequest) -> bool {
    pull_request.state == Some(IssueState::Closed)
}

pub fn should_update(pull_request: &PullRequest) -> bool {
    pull_request.mergeable_state == Some(MergeableState::Behind)
}

pub fn should_merge(pull_request: &PullRequest) -> bool {
    if pull_request.mergeable.unwrap() == false {
        return false;
    }

    if pull_request.mergeable_state == Some(MergeableState::Clean) {
        true
    } else {
        false
    }
}
