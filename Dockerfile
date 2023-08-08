FROM clux/muslrust:stable as builder

WORKDIR /app

ADD ./src ./src
ADD ./Cargo.lock ./
ADD ./Cargo.toml ./

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine AS runtime
RUN addgroup -S action-runner && adduser -S action-runner -G action-runner

RUN mkdir -p /app
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ /app/

USER action-runner

#ENV GITHUB_SERVER_URL
#ENV GITHUB_REPOSITORY
#ENV GITHUB_SHA

ENTRYPOINT ["/app/deploy-register-artifact-action"]