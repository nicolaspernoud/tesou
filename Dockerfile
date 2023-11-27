###########################
# Stage 1 : Backend build #
###########################

FROM rust:1.74 as backend-builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=appuser
ENV UID=1000
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /build

COPY ./backend .

RUN cargo test
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN mkdir -p /app/db/
RUN chown -Rf "${UID}":"${UID}" /app/db/
RUN mkdir -p /app/data/
RUN chown -Rf "${UID}":"${UID}" /app/data/

############################
# Stage 2 : Frontend build #
############################

FROM ghcr.io/cirruslabs/flutter:3.16.0 as frontend-builder
WORKDIR /build
COPY ./frontend .
RUN flutter pub get
RUN flutter test
RUN flutter build web

#########################
# Stage 3 : Final image #
#########################

FROM scratch

COPY --from=backend-builder /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=backend-builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=backend-builder /etc/passwd /etc/passwd
COPY --from=backend-builder /etc/group /etc/group

WORKDIR /app
COPY --from=backend-builder /app ./
COPY --from=backend-builder /build/target/x86_64-unknown-linux-musl/release/tesou ./
COPY --from=frontend-builder /build/build/web/ /app/web/

USER appuser:appuser
ENTRYPOINT ["./tesou"]
