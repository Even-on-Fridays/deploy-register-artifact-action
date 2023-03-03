use std::ffi::OsStr;
use std::process::exit;
use anyhow::anyhow;
use clap::Parser;
use graphql_client::GraphQLQuery;
use reqwest::blocking::Response;
use reqwest::StatusCode;
use crate::git_hub_action_register_docker_image_push::{GitRepositoryProvider, RegisterDockerImagePushInput};

fn main() {
    let (input, env) = match read_input() {
        Ok((input, env)) => (input, env),
        Err(err) => return print_and_exit(format!("Invalid input: {err:?}")),
    };

    let config = match Config::try_from_env() {
        Ok(c) => c,
        Err(err) => return print_and_exit(format!("Invalid input: {err:?}")),
    };

    let vars = git_hub_action_register_docker_image_push::Variables{ input: RegisterDockerImagePushInput {
        docker_image_ref: input.docker_image,
        git_repository_provider: GitRepositoryProvider::GIT_HUB,
        git_repository_server_url: env.server_url,
        git_repository_full_name: env.repository,
        commit_hash: env.commit_sha,
    } };

    if let Err(err) = call_api(config, vars) {
        print_and_exit(format!("Failed to register docker image push: {err:?}"));
    }
}

fn print_and_exit(message: String) {
    eprintln!("{message}.");
    exit(1);
}

fn call_api(config: Config, variables: git_hub_action_register_docker_image_push::Variables) -> anyhow::Result<()> {
    let request_body = GitHubActionRegisterDockerImagePush::build_query(variables);

    let client = reqwest::blocking::Client::new();
    let mut res = client.post(config.graphql_url).header("authorization", "FAKE:acme").json(&request_body).send()?;

    if res.status() != StatusCode::OK {
        return Err(anyhow!("server responded with http status {:?}", res.status()));
    }

    let response_body: graphql_client::Response<git_hub_action_register_docker_image_push::ResponseData> = res.json()?;

    if let Some(errors) = response_body.errors {
        return Err(anyhow!("server respond: {}", errors.iter().map(|err|err.to_string()).collect::<Vec<_>>().join(" / ")));
    }

    Ok(())
}

fn read_input() -> anyhow::Result<(InputParams, EnvParams)>{
    let args = InputParams::parse();
    let params = EnvParams::try_from_env()?;

    Ok((args, params))
}

#[derive(GraphQLQuery)]
#[graphql(
schema_path = "schema.graphql",
query_path = "register_docker_image_push.graphql",
)]
struct GitHubActionRegisterDockerImagePush;

struct Config {
    graphql_url: reqwest::Url,
}

impl Config {
    fn try_from_env() -> anyhow::Result<Self> {
        Ok(Config{
            graphql_url: std::env::var("T3_GRAPHQL_URL").ok().unwrap_or_else(||String::from("https://api.transistor.eof.dev/graphql")).as_str().try_into()?,
        })
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct InputParams {
    #[arg(short, long)]
    docker_image: String,
}

struct EnvParams {
    server_url: String,
    repository: String,
    commit_sha: String,
}

impl EnvParams {
    fn try_from_env() -> anyhow::Result<Self> {
        Ok(EnvParams{
            server_url: read_env("GITHUB_SERVER_URL")?,
            repository: read_env("GITHUB_REPOSITORY")?,
            commit_sha: read_env("GITHUB_SHA")?,
        })
    }
}

fn read_env<K: AsRef<OsStr>>(var: K) -> anyhow::Result<String> {
    std::env::var(var.as_ref()).map_err(|_|anyhow!("missing environment variable {:?}", var.as_ref().to_str()))
}