# Use the official Rust image as a base
FROM rust:slim-bullseye

RUN apt-get update && apt-get -y install \
    cron libssl-dev pkg-config

COPY . /app/

# Set the working directory inside the container
WORKDIR /app/src

# Build your application
RUN cargo build --release

# Add your cron job
RUN echo "*/1 * * * * root cd /app && ./target/release/HardwareScrape > /proc/1/fd/1 2>/proc/1/fd/2" >> /etc/crontab

# Start the cron daemon
CMD cron -f
