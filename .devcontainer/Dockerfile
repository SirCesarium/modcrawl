ARG VARIANT=trixie

FROM mcr.microsoft.com/devcontainers/rust:${VARIANT}

RUN apt-get update \
	&& apt-get install -y --no-install-recommends fd-find ripgrep \
	&& ln -sf /usr/bin/fdfind /usr/local/bin/fd \
	&& rm -rf /var/lib/apt/lists/*

RUN rustup update stable \
	&& rustup default stable \
	&& rustup component add clippy rustfmt

RUN rustc --version && cargo --version && fd --version && rg --version
