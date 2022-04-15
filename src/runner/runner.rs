use super::Action;
use super::{should_merge, should_pop, should_update};
use crate::config::{Configuration, Notification};
use crate::pull_request::PullRequest;
use anyhow::{bail, Result};
use log::{info, trace, warn};
use notify_rust::Timeout;
use octocrab::Octocrab;

pub struct Runner {
    configuration: Configuration,
    requests_to_remove: Vec<PullRequest>,
}

impl Runner {
    pub fn new(configuration: Configuration) -> Result<Self> {
        if let Ok(token) = std::env::var("GITHUB_API_TOKEN") {
            octocrab::initialise(Octocrab::builder().personal_token(token))?;

            Ok(Self {
                configuration,
                requests_to_remove: Vec::new(),
            })
        } else {
            bail!("Invalid GITHUB_API_TOKEN")
        }
    }

    pub async fn process(&mut self, pull_requests: Vec<PullRequest>) -> Result<()> {
        let octocrab = octocrab::instance();
        info!("Processing queue");

        for pull_request in pull_requests {
            let action: Action = self.detect_action(&pull_request).await?;

            let to_remove: Option<PullRequest> = match action.clone() {
                Action::NoOp => None,
                Action::Pop => Some(pull_request.clone()),
                Action::Update => {
                    info!("Updating: {:?}", &pull_request.url);

                    octocrab
                        .pulls(&pull_request.owner, &pull_request.repo)
                        .update_branch(pull_request.request)
                        .await?;

                    None
                }
                Action::Merge => {
                    info!("Merging: {:?}", &pull_request.url);
                    // TODO: Extract
                    let pull =
                        octocrab.pulls(&pull_request.owner.clone(), &pull_request.repo.clone());
                    let mut merge_call = pull
                        .merge(pull_request.request)
                        .method(octocrab::params::pulls::MergeMethod::Squash);

                    if let Some(title) = &self.configuration.merger.title {
                        merge_call = merge_call.title(title);
                    }

                    if let Some(message) = &self.configuration.merger.message {
                        merge_call = merge_call.message(message);
                    }

                    merge_call.send().await?;

                    Some(pull_request.clone())
                }
            };

            self.dispatch_message(action, &pull_request);

            if let Some(request) = to_remove {
                self.requests_to_remove.push(request);
            }
        }

        Ok(())
    }

    pub fn dispatch_message(&self, action: Action, pull_request: &PullRequest) {
        let option: &Option<Notification> = match action {
            Action::NoOp => {
                return;
            }
            Action::Merge => &self.configuration.notifier.merge,
            Action::Update => &self.configuration.notifier.update,
            Action::Pop => &self.configuration.notifier.pop,
        };

        if let Some(notification) = option {
            if !notification.enabled {
                return;
            }

            let mut builder = notify_rust::Notification::new();

            let mut handler = builder
                .summary(&notification.title)
                .body(
                    &notification
                        .message
                        .as_ref()
                        .unwrap_or(&format!("Notification for {:?}", action)),
                )
                .timeout(Timeout::Milliseconds(6000));

            if let Some(icon) = &notification.icon {
                handler = handler.icon(icon);
            }

            handler.show().unwrap();
        }
    }

    async fn detect_action(&mut self, pull_request: &PullRequest) -> Result<Action> {
        let octocrab = octocrab::instance();
        let pr = octocrab
            .pulls(&pull_request.owner, &pull_request.repo)
            .media_type(octocrab::params::pulls::MediaType::Full)
            .get(pull_request.request)
            .await?;

        if self::should_pop(&pr) {
            return Ok(Action::Pop);
        }

        if self::should_update(&pr) {
            return Ok(Action::Update);
        }

        if self::should_merge(&pr) {
            return Ok(Action::Merge);
        }

        Ok(Action::NoOp)
    }

    pub fn cleanup(&self, pull_requests: &mut Vec<PullRequest>) {
        for pull_request in &self.requests_to_remove {
            let index = pull_requests
                .iter()
                .position(|p| (*p).same(pull_request))
                .unwrap();

            pull_requests.remove(index);
        }
    }
}
