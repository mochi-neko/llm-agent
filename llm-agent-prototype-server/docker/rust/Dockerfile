# Base image for CUDA 11.8 and cuDNN 8
FROM nvidia/cuda:11.8.0-cudnn8-devel-ubuntu22.04

# Install packages with cleanup
RUN apt-get update \
    && apt-get install -y \
    build-essential \
    wget \
    unzip \
    curl \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    libprotobuf-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Rust 1.71.1
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && $HOME/.cargo/bin/rustup install 1.71.1 \
    && $HOME/.cargo/bin/rustup default 1.71.1
# Set environment variables for Ruut
ENV PATH="/root/.cargo/bin:${PATH}"
# Setup Rust components and tools
RUN rustup component add \
    rls \
    rust-analysis \
    rust-src \
    rustfmt \
    clippy \
    && cargo install \
    cargo-edit \
    cargo-watch
# Set environment variables for Rust logging
ENV RUST_LOG=info

# Install grpcurl
RUN cd /tmp/ && wget https://github.com/fullstorydev/grpcurl/releases/download/v1.8.0/grpcurl_1.8.0_linux_x86_64.tar.gz && \
    tar -xf grpcurl_1.8.0_linux_x86_64.tar.gz && \
    mv grpcurl /usr/local/bin/

# Download and install libtorch 2.0.1 for CUDA 11.8
ENV LIBTORCH_VERSION=2.0.1
RUN wget https://download.pytorch.org/libtorch/cu118/libtorch-cxx11-abi-shared-with-deps-${LIBTORCH_VERSION}%2Bcu118.zip \
    && unzip libtorch-cxx11-abi-shared-with-deps-${LIBTORCH_VERSION}+cu118.zip -d /opt \
    && rm libtorch-cxx11-abi-shared-with-deps-${LIBTORCH_VERSION}+cu118.zip
# Set environment variables for libtorch
ENV TORCH_LIB_PATH=/opt/libtorch/lib
ENV LD_LIBRARY_PATH=$TORCH_LIB_PATH:$LD_LIBRARY_PATH
ENV LIBTORCH=/opt/libtorch

# Set working directory
WORKDIR /workspace

# Open port for gRPC
EXPOSE 50051