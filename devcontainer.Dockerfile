FROM debian:12

RUN apt update && apt install -y curl build-essential pkg-config libssl-dev openssh-server vim

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Add cargo binaries to path
ENV PATH="/usr/local/cargo/bin:${PATH}"

# Create workspace directory
WORKDIR /workspace

# Pre-install common Rust development tools
RUN rustup component add rustfmt clippy

RUN cargo install sqlx-cli

# Ensure target directory exists and has right permissions
RUN mkdir -p /workspace/target

# Configure SSH server
RUN mkdir -p /var/run/sshd
RUN echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config
RUN echo 'PasswordAuthentication no' >> /etc/ssh/sshd_config
RUN mkdir -p /root/.ssh

# Ensure the container doesn't exit
EXPOSE 22
