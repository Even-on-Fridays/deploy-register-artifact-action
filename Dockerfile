FROM clux/muslrust:stable as builder

WORKDIR /app

ADD ./src ./src
ADD ./Cargo.lock ./
ADD ./Cargo.toml ./
ADD ./*.graphql ./

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine AS runtime
RUN addgroup -S transistor && adduser -S transistor -G transistor

RUN mkdir -p /app
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ /app/

USER transistor

#ENV GITHUB_SERVER_URL
#ENV GITHUB_REPOSITORY
#ENV GITHUB_SHA

ENTRYPOINT ["/app/transistor-register-artifact-action"]