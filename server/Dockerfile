# Contexte de build : server/. Secrets et PIAF_BBOX viennent de l'environnement, voir docs/02-architecture.md.
FROM rust:1.99-trixie AS build
# aws-lc-sys, tiré par rustls, compile du C.
RUN apt-get update && apt-get install -y --no-install-recommends cmake && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# ponytail: pas de cache des dépendances, chaque build recompile tout. À revoir si les déploiements deviennent fréquents.
RUN cargo build --release --locked

FROM debian:trixie-slim
# rustls lit les autorités de certification du système.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/is-it-raining-server /usr/local/bin/
USER nobody
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
CMD ["is-it-raining-server"]
