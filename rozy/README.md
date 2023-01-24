This is a Rust implementation of Ozy.

## Instructions
1. Install [`rustup`](https://rustup.rs/)
2. Run `install.sh`
4. Prepend `$HOME/.ozy/bin` to your path to have access to managed apps

Performance is really only a consideration on the common case of running a ozy-managed binary. This is why we prefer not to introduce asynchrony or even template caching in the `install-all` path.

## Currently implemented commands
* `ozy clean` deletes managed directories
* `ozy init` sets up app symlinks from a base config from a provided URL
* `ozy update` accepts an optional URL, but will use the init-configured one without one
* `ozy install [app name]` installs a single app
* `ozy install-all` installs them all
* `ozy run [app name]` will run that app. Typically, you'll just run `[app name]` in the shell directly
* `ozy makefile-config` output `Makefile` compatible configuration for a list of apps
