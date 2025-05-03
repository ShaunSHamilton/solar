# Use a stable Debian base image - good foundation for dev tools and services
FROM debian:stable-slim AS builder

# Avoid prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install prerequisites including process management tools
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        apt-transport-https \
        ca-certificates \
        curl \
        gnupg \
        lsb-release \
        build-essential \
        pkg-config \
        libssl-dev \
        git \
        sudo \
        unzip \
        procps && \
    rm -rf /var/lib/apt/lists/*

# === Install InfluxDB v1 ===
# Add InfluxData repository key
RUN curl -s https://repos.influxdata.com/influxdata-archive_compat.key | gpg --dearmor | tee /etc/apt/trusted.gpg.d/influxdata-archive_compat.gpg > /dev/null
# Add InfluxData repository
RUN echo 'deb [signed-by=/etc/apt/trusted.gpg.d/influxdata-archive_compat.gpg] https://repos.influxdata.com/debian stable main' | tee /etc/apt/sources.list.d/influxdb.list

# Install InfluxDB v1 (latest 1.x series)
# influxdb package automatically creates the 'influxdb' user/group and /var/lib/influxdb directory
RUN apt-get update && \
    apt-get install -y --no-install-recommends influxdb && \
    rm -rf /var/lib/apt/lists/*

# === Install Rust ===
# Install rustup (Rust toolchain manager) and the latest stable Rust toolchain non-interactively
# Also add common targets and components useful for development
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile default \
    && chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# === Setup Development User ===
# Create a non-root user 'vscode' (common for devcontainers) with sudo access
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME && \
    useradd --uid $USER_UID --gid $USER_GID -m $USERNAME --shell /bin/bash && \
    echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME && \
    chmod 0440 /etc/sudoers.d/$USERNAME

# === Copy Data ===
# Copy the data file into the standard InfluxDB data directory.
COPY ../data/wattle-i-do.zip /var/lib/influxdb/wattle-i-do.zip

# === Configure Environment ===
# Default InfluxDB port
EXPOSE 8086

# Define the InfluxDB data directory as a volume mount point.
VOLUME /var/lib/influxdb

# Switch to the non-root user
USER $USERNAME

# Set a working directory for development projects
WORKDIR /workspace

# Set PATH again for the non-root user
ENV PATH=$CARGO_HOME/bin:$PATH

# Default command - useful for devcontainers to keep the container running.
# The actual command/entrypoint might be overridden by devcontainer.json settings.
CMD ["/bin/bash"]