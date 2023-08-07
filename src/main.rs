use anyhow::anyhow;
use clap::Parser;
use reqwest::StatusCode;
use serde::Serialize;
use std::ffi::OsStr;
use std::process::exit;

fn main() {
    let (input, env) = match read_input() {
        Ok((input, env)) => (input, env),
        Err(err) => return print_and_exit(format!("Invalid input: {err:?}")),
    };

    let config = match Config::try_from_env() {
        Ok(c) => c,
        Err(err) => return print_and_exit(format!("Invalid input: {err:?}")),
    };

    let vars = GithubActionArtifactPushPayload {
        org_id: input.org_id,
        docker_image_ref: input.docker_image,
        git_repository_provider: GitRepositoryProvider::GitHub,
        git_repository_server_url: env.server_url,
        git_repository_full_name: env.repository,
        commit_hash: env.commit_sha,
    };

    if let Err(err) = call_api(config, vars) {
        print_and_exit(format!("Failed to register docker image push: {err:?}"));
    }
}

#[derive(Serialize)]
struct GithubActionArtifactPushPayload {
    org_id: String,
    docker_image_ref: String,
    git_repository_provider: GitRepositoryProvider,
    git_repository_server_url: String,
    git_repository_full_name: String,
    commit_hash: String,
}

#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GitRepositoryProvider {
    GitHub,
}

fn print_and_exit(message: String) {
    eprintln!("{message}.");
    exit(1);
}

fn call_api(config: Config, request_body: GithubActionArtifactPushPayload) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(
            config
                .transistor_api_base_url
                .join("/connector/webhook/github-actions/artifact-push")?,
        )
        .json(&request_body)
        .send()?;

    if res.status() != StatusCode::OK {
        return Err(anyhow!(
            "server responded with http status {:?}",
            res.status()
        ));
    }

    Ok(())
}

fn read_input() -> anyhow::Result<(InputParams, EnvParams)> {
    let args = InputParams::parse();
    let params = EnvParams::try_from_env()?;

    Ok((args, params))
}

struct Config {
    transistor_api_base_url: reqwest::Url,
}

impl Config {
    fn try_from_env() -> anyhow::Result<Self> {
        Ok(Config {
            transistor_api_base_url: std::env::var("EOF_DEPLOY_BASE_URL")
                .ok()
                .unwrap_or_else(|| String::from("https://deploy.eofsuite.com"))
                .as_str()
                .try_into()?,
        })
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct InputParams {
    #[arg(short, long)]
    docker_image: String,

    #[arg(short, long)]
    org_id: String,
}

struct EnvParams {
    server_url: String,
    repository: String,
    commit_sha: String,
}

impl EnvParams {
    fn try_from_env() -> anyhow::Result<Self> {
        Ok(EnvParams {
            server_url: read_env("GITHUB_SERVER_URL")?,
            repository: read_env("GITHUB_REPOSITORY")?,
            commit_sha: read_env("GITHUB_SHA")?,
        })
    }
}

fn read_env<K: AsRef<OsStr>>(var: K) -> anyhow::Result<String> {
    std::env::var(var.as_ref())
        .map_err(|_| anyhow!("missing environment variable {:?}", var.as_ref().to_str()))
}
