#syntax=docker/dockerfile:1

##
## Build
##
FROM rust:1-alpine3.20 AS build
LABEL org.opencontainers.image.authors="Sangbum Kim <sangbumkim@amuz.es>"

# set the workdir and copy the source into it
WORKDIR /app
COPY . /app


ENV RUSTFLAGS='-Cpanic=abort -Crelocation-model=static -Clink-args=-Wl,-x,-s,-fuse-ld=lld,--as-needed,--gc-sections,--no-gnu-unique,--nostdlib,--no-pie,--build-id=none,--no-eh-frame-hdr'

RUN set -x && \
    apk add --no-cache \
        libcap-static \
        libcap-dev \
        lld \
        musl-dev

RUN set -x && \
    cargo build --release && \
    objcopy -R .eh_frame -R .got.plt target/release/init-wrapper target/release/init-wrapper && \
    ls -alh target/release/init-wrapper && \
    readelf -W -S  ./target/release/init-wrapper

RUN --mount=type=bind,rw,source=.,target=/host \
    cp -avf target/release/init-wrapper /host/init-wrapper
#     cp -avf target/release/init-wrapper /host/init-wrapper && \
#     ./target/release/init-wrapper


FROM scratch
COPY --from=build /app/target/release/init-wrapper /init-wrapper


# ENTRYPOINT ['/init-wrapper']