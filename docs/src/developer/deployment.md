# Deployment

## Editor Deployment Pipeline

- Latest commit on the ``main`` branch automatically
  triggers [``cd-editor-staging.yml``](https://github.com/rdeepak2002/dream-engine/blob/5-python-script-component-system/.github/workflows/cd-editor-staging.yml)
  GitHub action to deploy to
  Netlify.

- Latest tagged release automatically
  triggers [``cd-editor-prod.yml``](https://github.com/rdeepak2002/dream-engine/blob/5-python-script-component-system/.github/workflows/cd-editor-prod.yml)
  GitHub action to deploy to Netlify.

- Latest commit on the ``main`` branch automatically
  triggers [``ci.yml``](https://github.com/rdeepak2002/dream-engine/blob/5-python-script-component-system/.github/workflows/ci.yml)
  GitHub action to verify Dream can be
  built on different systems.

## Docs Deployment Pipeline

Latest commit in ``docs`` folder on ``main`` branch automatically
triggers [``deploy-docs.yml``](https://github.com/rdeepak2002/dream-engine/blob/5-python-script-component-system/.github/workflows/cd-docs.yml)
GitHub action to deploy
to Netlify.

## Domain Management

The ``dream-engine.app`` domain is managed by Google Domains.