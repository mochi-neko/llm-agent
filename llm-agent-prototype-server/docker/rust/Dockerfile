FROM rust:1.71-buster

RUN apt-get update && \
    apt-get -y install git protobuf-compiler libprotobuf-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    rustup component add rls rust-analysis rust-src rustfmt clippy && \
    cargo install cargo-edit cargo-watch

# Install grpcurl
RUN cd /tmp/ && wget https://github.com/fullstorydev/grpcurl/releases/download/v1.8.0/grpcurl_1.8.0_linux_x86_64.tar.gz && \
    tar -xf grpcurl_1.8.0_linux_x86_64.tar.gz && \
    mv grpcurl /usr/local/bin/

EXPOSE 8000