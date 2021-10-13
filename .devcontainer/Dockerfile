# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.187.0/containers/codespaces-linux/.devcontainer/base.Dockerfile

FROM mcr.microsoft.com/vscode/devcontainers/universal:1-focal

USER root

RUN apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends tldr gcc-arm-linux-gnueabihf

RUN cargo install bat ripgrep cargo-deb

USER codespace

RUN rustup target add armv7-unknown-linux-gnueabihf
