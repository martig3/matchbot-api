# matchbot-api

Web server that handles integrations for [csgo-matchbot](https://github.com/martig3/csgo-matchbot)

## Dathost Webhooks

Supports the following webhooks & associated features:

- `/match-end` - handles match end logic & auto demo upload to S3 compatible bucket
- `/series-end` - handles series end logic & auto demo upload to S3 compatible bucket
- `/round-end` - handles round end score updates

## Usage

Set following env:

```dotenv
ENV=<prd or dev>
PORT=8080
RUST_LOG=matchbot-api=info
DATHOST_USER=<dathost username/email>
DATHOST_PASSWORD=<dathost password>
DATABASE_URL=postgres://postgres:postgres@localhost/matchbot
AWS_ACCESS_KEY_ID=<aws access key>
AWS_SECRET_ACCESS_KEY=<aws secret key>
BUCKET_NAME=<s3 bucketname>
AWS_ENDPOINT=<s3 endpoint (I made this to use cloudflare s2, so unless you manually provide the aws endpoint it won't work. Submit an feature request if someone wants this!>
TV_DELAY=<csgo server tv_delay value, defaults to 105>
STEAM_API_KEY=<steam web api key>
```

`docker run --env-file .env -d -p 8080:8080 ghcr.io/martig3/matchbot-api:latest`
