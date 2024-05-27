# nixpkgs-tracker-bot

A small Discord ~~bot~~ app(!) that helps you track where [nixpkgs](https://github.com/NixOS/nixpkgs) PRs have reached

## Currently supported branches

- master
- staging
- nixos-unstable
- nixos-unstable-small
- nixos-24.05-small
- release-24.05
- nixos-23.11-small
- release-23.11

## TODO

- [ ] Cache responses (to avoid rate limiting)
- [ ] Allow for authenticated requests to GH (to avoid rate limiting)
- [ ] Don't make so many API requests for each invocation (to avoid rate limiting...this is a problem see?)
- [ ] Switch to poise after https://github.com/serenity-rs/poise/pull/266
