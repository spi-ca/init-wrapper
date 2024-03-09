#syntax=docker/dockerfile:1

##
## Build
##
FROM rust:1-alpine3.19 AS build
LABEL org.opencontainers.image.authors="Sangbum Kim <sangbumkim@amuz.es>"

# set the workdir and copy the source into it
WORKDIR /app
COPY . /app

ENV RUSTFLAGS='-C panic=abort -C link-arg=-s -C link-arg=-fuse-ld=lld'

RUN set -x && \
    apk add --no-cache \
        libcap-static \
        libcap-dev \
        lld \
        musl-dev

RUN set -x && \
    cargo build --release && \
    ls -alh target/release/init-wrapper
    # ldd target/release/init-wrapper && \
    # && \
    # ldd target/release/init-wrapper

# RUN --mount=type=bind,rw,source=.,target=/host \
#     cp -avf target/release/init-wrapper /host/init-wrapper && \
#     ./target/release/init-wrapper


FROM scratch
COPY --from=build /app/target/release/init-wrapper /init-wrapper


# ENTRYPOINT ['/init-wrapper']