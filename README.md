# action-release-notifier

action-release-notifier is a github action which is used to notify whenever there is a new upstream github repo releases.

## Motivation

Since most of applications these days rely on upstream opensource Github Projects, we needed a way to get notificed when a new release of upstream was done and easily configured to any number of github projects using Github Actions.

## Build status

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


