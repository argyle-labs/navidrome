# TODO: base image + build for navidrome. Mirror jellyfin/Dockerfile conventions.
FROM debian:12-slim
LABEL org.opencontainers.image.source="https://github.com/argyle-labs/navidrome"
EXPOSE 4533
