FROM rust:alpine AS build

RUN apk add --no-cache build-base && mkdir -p /app
COPY . /app
WORKDIR /app
RUN cargo build --release && strip target/release/matchbot-api

FROM scratch
COPY --from=build /app/target/release/matchbot-api .
CMD [ "/matchbot-api" ]
