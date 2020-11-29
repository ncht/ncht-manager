FROM rust:1.48 as builder

WORKDIR /work
COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc

WORKDIR /app

COPY --from=builder /work/target/release/ncht-manager /app

CMD [ "/app/ncht-manager" ]
