# To create a docker image we can use for cross-compilation
FROM ubuntu:20.04

# Get our dependencies
RUN apt update
RUN apt-get install -y curl build-essential libclang-dev unzip gcc-multilib

# libclang ENV
ENV LIBCLANG_PATH /lib/llvm-10/lib/

# Grab and unzip the Android NDK
RUN mkdir /ndk
RUN curl -o /tmp/ndk.zip https://dl.google.com/android/repository/android-ndk-r21d-linux-x86_64.zip
# COPY ./android-ndk-r21d-linux-x86_64.zip /tmp/ndk.zip
RUN unzip /tmp/ndk.zip -d /ndk
ENV ANDROID_NDK_HOME /ndk/android-ndk-r21d

# Install Rust
RUN mkdir -p /rust/rustup
RUN mkdir /rust/cargo

ENV RUSTUP_HOME /rust/rustup
ENV CARGO_HOME /rust/cargo
ENV PATH $PATH:$CARGO_HOME/bin
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Rust Android stuff
RUN rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
RUN cargo install cargo-ndk

ENTRYPOINT ["cargo", "ndk"]