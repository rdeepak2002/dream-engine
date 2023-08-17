# Deployment

## Pipeline

The entire deployment pipeline is managed via GitHub actions present in the ``.github/workflows`` folder and
a [Heroku pipeline](https://dashboard.heroku.com/pipelines/2e75564a-04b9-4c90-8b12-cc6add0e68aa)

For GitHub Actions, a ``HEROKU_API_KEY`` secret environment variable must be defined. The API key for Heroku can be
retrieved from
the [user's account page](https://dashboard.heroku.com/account)

For deploying please follow this pipeline:

- Latest commit on ``main`` branch is automatically deployed
  to [dev Heroku app](https://dashboard.heroku.com/apps/dream-rs-dev)
- Latest tagged release is automatically deployed to [test Heroku app](https://dashboard.heroku.com/apps/dream-rs-test)
- Release of [prod Heroku app](https://dashboard.heroku.com/apps/dream-rs-prod) is done by promoting test to prod on
  the [Heroku pipeline UI](https://dashboard.heroku.com/pipelines/2e75564a-04b9-4c90-8b12-cc6add0e68aa)
