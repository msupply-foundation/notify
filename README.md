# Notify

"Notify" is a multi channel notification scheduling tool.

To read more about what it does and how to use it, visit our documentation site: https://docs.msupply.foundation/notify/introduction/

## Setup

You'll need to install the following tools

- [git](https://git-scm.com/)
- [Rust](https://www.rust-lang.org/tools/install)
- [diesel_cli](https://crates.io/crates/diesel_cli)
- [nvm](https://github.com/nvm-sh/nvm#installing-and-updating) to manage your node version conveniently (with .nvmrc)
  - Or install [node.js](https://nodejs.org/), preferably the version in `frontend/.nvmrc`
- [Yarn](https://yarnpkg.com/getting-started/install)

You'll also need to create a Telegram bot and configure the token in `configuration/local.yaml`. Please follow the instructions in [this](https://github.com/msupply-foundation/notify/edit/main/backend/README.md) document.

### Linux

- For Ubuntu:

```bash
  sudo apt-get install libpq-dev
```

### Mac

- For M1 Mac:

`brew install libpq` and add the following to `~/.cargo/config.toml`

```
[env]
MACOSX_DEPLOYMENT_TARGET = "10.7"

[target.aarch64-apple-darwin]
rustflags = "-L /opt/homebrew/opt/libpq/lib"
```

You may also need to run
`brew link --force libpq`

## Running Notify locally

To get the application running locally on your machine, run the following commands:

### 1. Build and start the backend

```bash
# In ./backend/
cargo run # Downloads dependencies, compiles and starts the backend server
```

### 2. Build the frontend

```bash
# In ./frontend/
nvm use # To use the node version specified in .nvmrc
yarn # Downloads dependencies

yarn start # Transpiles the frontend artifacts to default location for backend server to serve
```

All going well, this should open the web app and login page in your default browser. If running for the first time (or you've deleted the database) the database will be initialised with a user to login with _username:_ `admin`, _password:_ `pass`.

## Development

Find further details about frontend and backend development in the respective README.md files:

- https://github.com/openmsupply/notify/blob/main/backend/README.md
- https://github.com/openmsupply/notify/blob/main/frontend/README.md

After setting up your environment, There are some quick start tutorials you can follow [here](./docs/tutorial/0.tutorials.md)

## Docs for setting up notifications

Until a documentation website is established, you can find some docs for setting up notify & notifications [here](./docs/notification_setup.md)

## Frequently Asked Questions

See the [FAQ here](./docs/faq.md)

## Contributing

To contribute, there are several key areas

1. Create issues for bugs or improvements to the system.
2. Taking on issues from our triaged list of issues. Please ask the team!
3. Create a pull request for your changes into the branch `main`
4. Review PRs of others ❤️

### Issues

Please use our issue templates. They help by providing a concise framework for including the minimum useful amount of information and some default labeling.

If working on an issue, assign yourself to the issue. Not doing so risks multiple people working on the same problem.

### Branching

> For now, we only have a `main` branch that get active development merged into it. In the future we may move to the below branching strategy to match other mSupply product workflows.

Our goal is to achieve something inspired by [gitflow](https://nvie.com/posts/a-successful-git-branching-model/) ([also a good link](http://datasift.github.io/gitflow/IntroducingGitFlow.html)). We have 2 protected branches:

- **main** is considered the stable, production ready branch. It may receive hotfixes.
- **develop** is where features and non-critical fixes are merged into.

When working on an issue you should create a branch on which you'll commit the changes you are making. Including the issue number in your branch name helps us identify which issue a particular branch relates to:

`X-fixes-thing-being-fixed`, where `X` is the issue number.

### Pull Requests

When the changes in your branch are ready:

1. Ensure backend tests pass by running `cargo test`.
1. Ensure frontend tests pass by running `yarn test`.
1. Make a pull request into the branch **develop**. Another person must review your changes by sanity checking the code and solution, and ideally give it a quick test in a dev environment.
1. Once approved, a reviewer might merge immediately or leave for you to merge (if sufficient permissions) if there are some optional suggestions to consider.

For further guidance on pull requests and review, see our detailed [Code Review guide](https://github.com/openmsupply/open-msupply/wiki/Code-Review-Comments---General)

### Release

When a set of changes in develop are considered stable, we'll merge develop into main and create a tag marking the release.

TMF staff can publish the change using the [documentation in our wiki](https://wiki.sussol.net/doku.php/health_supply_hub:config)

## Key Design Decisions (KDD)

Important Design Decisions for this project will be documented in the repository in docs/decisions.
The approach is similar to the [design decision log](https://microsoft.github.io/code-with-engineering-playbook/design/design-reviews/decision-log/) from Microsoft.
