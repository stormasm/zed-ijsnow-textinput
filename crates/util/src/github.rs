use crate::http::HttpClient;
use anyhow::{anyhow, bail, Context, Result};
use futures::AsyncReadExt;
use serde::Deserialize;
use std::sync::Arc;
use url::Url;

pub struct GitHubLspBinaryVersion {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct GithubRelease {
    pub tag_name: String,
    #[serde(rename = "prerelease")]
    pub pre_release: bool,
    pub assets: Vec<GithubReleaseAsset>,
    pub tarball_url: String,
    pub zipball_url: String,
}

#[derive(Deserialize, Debug)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub async fn latest_github_release(
    repo_name_with_owner: &str,
    require_assets: bool,
    pre_release: bool,
    http: Arc<dyn HttpClient>,
) -> Result<GithubRelease, anyhow::Error> {
    let mut response = http
        .get(
            &format!("https://api.github.com/repos/{repo_name_with_owner}/releases"),
            Default::default(),
            true,
        )
        .await
        .context("error fetching latest release")?;

    let mut body = Vec::new();
    response
        .body_mut()
        .read_to_end(&mut body)
        .await
        .context("error reading latest release")?;

    if response.status().is_client_error() {
        let text = String::from_utf8_lossy(body.as_slice());
        bail!(
            "status error {}, response: {text:?}",
            response.status().as_u16()
        );
    }

    let releases = match serde_json::from_slice::<Vec<GithubRelease>>(body.as_slice()) {
        Ok(releases) => releases,

        Err(err) => {
            log::error!("Error deserializing: {:?}", err);
            log::error!(
                "GitHub API response text: {:?}",
                String::from_utf8_lossy(body.as_slice())
            );
            return Err(anyhow!("error deserializing latest release"));
        }
    };

    releases
        .into_iter()
        .filter(|release| !require_assets || !release.assets.is_empty())
        .find(|release| release.pre_release == pre_release)
        .ok_or(anyhow!("Failed to find a release"))
}

pub async fn github_release_with_tag(
    repo_name_with_owner: &str,
    tag: &str,
    http: Arc<dyn HttpClient>,
) -> Result<GithubRelease, anyhow::Error> {
    let url = build_tagged_release_url(repo_name_with_owner, tag)?;
    let mut response = http
        .get(&url, Default::default(), true)
        .await
        .with_context(|| format!("error fetching release {} of {}", tag, repo_name_with_owner))?;

    let mut body = Vec::new();
    response
        .body_mut()
        .read_to_end(&mut body)
        .await
        .with_context(|| {
            format!(
                "error reading response body for release {} of {}",
                tag, repo_name_with_owner
            )
        })?;

    if response.status().is_client_error() {
        let text = String::from_utf8_lossy(body.as_slice());
        bail!(
            "status error {}, response: {text:?}",
            response.status().as_u16()
        );
    }

    match serde_json::from_slice::<GithubRelease>(body.as_slice()) {
        Ok(release) => Ok(release),

        Err(err) => {
            log::error!("Error deserializing: {:?}", err);
            log::error!(
                "GitHub API response text: {:?}",
                String::from_utf8_lossy(body.as_slice())
            );
            Err(anyhow!(
                "error deserializing release {} of {}",
                tag,
                repo_name_with_owner
            ))
        }
    }
}

fn build_tagged_release_url(repo_name_with_owner: &str, tag: &str) -> Result<String> {
    let mut url = Url::parse(&format!(
        "https://api.github.com/repos/{repo_name_with_owner}/releases/tags"
    ))?;
    // We're pushing this here, because tags may contain `/` and other characters
    // that need to be escaped.
    url.path_segments_mut()
        .map_err(|_| anyhow!("cannot modify url path segments"))?
        .push(tag);
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::build_tagged_release_url;

    #[test]
    fn test_build_tagged_release_url() {
        let tag = "release/2.2.20-Insider";
        let repo_name_with_owner = "microsoft/vscode-eslint";

        let have = build_tagged_release_url(repo_name_with_owner, tag).unwrap();

        assert_eq!(have, "https://api.github.com/repos/microsoft/vscode-eslint/releases/tags/release%2F2.2.20-Insider");
    }
}