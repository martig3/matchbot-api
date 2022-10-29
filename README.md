# matchbot-api

Web server that handles integrations for [csgo-matchbot](https://github.com/martig3/csgo-matchbot)

## Dathost Webhooks

Supports the following webhooks & associated features:

- `/match-end` - handles match end logic & auto demo upload to S3 compatible bucket
- `/round-end` - handles round end score updates