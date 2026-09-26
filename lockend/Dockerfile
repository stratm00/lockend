ARG RUSTVER=1.95.0
ARG APP_NAME=lockend

##BUILD STAGE
FROM docker.io/library/rust:${RUSTVER}-alpine AS build

ARG APP_NAME
WORKDIR /app
#Deps
RUN apk add --no-cache clang lld musl-dev git

#Build CMD
RUN --mount=type=bind,source=src,target=src \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=bind,source=.env,target=.env \
  --mount=type=bind,source=.sqlx,target=.sqlx \
  --mount=type=cache,target=/app/target \
  --mount=type=cache,target=/var/cache/cargo\
  SQLX_OFFLINE=true \
  DATABASE_URL=postgresql://postgres:pw@localhost:5432/locker \
  CARGO_HOME=/var/cache/cargo cargo build --locked --release && \
  cp ./target/release/$APP_NAME /bin/server && \
  cp .env /bin/.env


##RUNTIME STAGE
FROM docker.io/library/alpine:3.18 AS final

ARG UID=10001
RUN adduser --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "${UID}" \
  appuser

USER appuser

COPY --from=build /bin/server /bin/
COPY --from=build /bin/.env /bin/

EXPOSE 80
CMD ["/bin/server"]

