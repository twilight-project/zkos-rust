#############################
# Build stage for zkos
#############################
FROM rust:1.87.0 as builder-zkos
RUN USER=root apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install git curl g++ build-essential libssl-dev pkg-config && \
    apt-get -y install software-properties-common protobuf-compiler && \
    apt-get update

RUN ls -a
RUN ls -a
RUN git clone -b v0.1.0 https://github.com/twilight-project/zkos-rust.git
WORKDIR /zkos-rust
# RUN sed -i 's|http://0\.0\.0\.0:7000|http://nyks:7000|g' transactionapi/src/rpcserver/service.rs
RUN cargo build --release 

#############################
# Runtime stage
#############################
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates protobuf-compiler jq libpq5 && \
    rm -rf /var/lib/apt/lists/*
EXPOSE 3030
EXPOSE 2500
RUN useradd -m zkos
WORKDIR /home/zkos
COPY --from=builder-zkos ./zkos-rust/target/release/api_server /usr/local/bin/api_server
ENTRYPOINT ["api_server"]