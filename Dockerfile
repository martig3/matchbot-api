FROM rust:alpine AS build

RUN apk add --no-cache build-base openssl-dev libpq-dev && mkdir -p /app
COPY . /app
WORKDIR /app
RUN cargo build --release && strip target/release/matchbot-api

FROM scratch
COPY --from=build /app/target/release/matchbot-api .
CMD [ "/matchbot-api" ]
#FROM rust:1.60-slim
#RUN apt-get update && apt-get install openssl pkg-config libpq-dev -y
#WORKDIR /usr/src/matchbot-api
#COPY . .
#
#RUN cargo install --path .
#
#CMD ["matchbot-api"]
