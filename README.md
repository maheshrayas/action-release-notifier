
<h1 align="center">
  <p align="center">action-release-notifier</p>
</h1>

<div align="center">
  <a href="https://github.com/maheshrayas/action-release-notifier/actions" alt="Build"><img src="https://github.com/maheshrayas/action-release-notifier/workflows/build/badge.svg" /></a>
  <a href="https://github.com/maheshrayas/action-release-notifier/actions/workflows/lint.yaml" alt="Lint"><img src="https://github.com/maheshrayas/action-release-notifier/actions/workflows/lint.yaml/badge.svg" /></a>
  <a href="https://github.com/maheshrayas/action-release-notifier/commits/main" alt="last commit"><img src="https://img.shields.io/github/last-commit/maheshrayas/action-release-notifier?color=purple" /></a>
</div>

## Motivation

Since most of applications these days rely on upstream opensource Github Projects, we needed a way to get notificed when a new release of upstream was done and easily configured to any number of github projects using Github Actions.

## Configuration in GHA workflow

```bash
name: Release Notifier

on:
  workflow_dispatch: null
  schedule:
    # Scheduled everyday At 00:00
    - cron: '0 0 * * *'

jobs:
  clean:
    runs-on: ubuntu-latest
    steps:
      - name: release-notifier
        uses: maheshrayas/action-release-notifier@v1
        with:
          github_token: '${{ secrets.GITHUB_TOKEN }}'
          repo: 'https://github.com/kubernetes/kubernetes,https://github.com/kubernetes-sigs/kustomize,https://github.com/helm/helm,https://github.com/istio/istio' #examples
          days: 1 #optional field, default 1 day, , make sure you set the cron appropriately, Example if you want to check for release once in 7 days, set days: 7 and cron schedule to run once in 7 days
```

## Notification
 
* GH Issue : A Github Issue would be created in the configured repo stating about the new release that found.
* Slack Notification : TODO


## Credits

[Michael Fornaro](https://github.com/xUnholy) for the guidance and improvements.


