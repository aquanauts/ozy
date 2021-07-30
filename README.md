# ozy

`ozy` is a Python program that makes it easy for you and your team to share and collaborate using commonly used programs such as `vault`, `nomad`, or `conda` on Linux.

## Getting Started

`ozy` takes a configuration URL as a parameter - this is specific to your team and controls which applications are supported and at which versions.

To demonstrate `ozy`, we'll be using the sample configuration served directly from GitHub, along with the latest `ozy` binary built in releases. The sample config supports some of the [Hashicorp](https://www.hashicorp.com/) apps, and some other simple apps. There's a convenience single-step installer for this, suitable for piping directly into bash:

```bash
curl -sL https://raw.githubusercontent.com/aquanauts/ozy/main/conf/install.sh | bash
```

Behind the scenes this fetches a released `ozy` binary and executes `ozy init https://path/to/this/github/conf/sample-team-conf.yaml`.

This will cause `ozy` to be installed in `~/.ozy`. In this directory there are some `ozy.*.yaml` files, and a `bin` directory. As part of the installation `ozy` will check and tell you how to put `~/.ozy/bin` into your path. It will place magic symlinks there that let it masquerade as all the applications it can install.

To see what the installer is doing see [ozy init](#Initializing) - but in short it is configuring itself from an Aquatic-specific endpoint.

### Running a command
Assume that your installation has been configured to support Hashicorp's [nomad](https://www.nomadproject.io/). One simply runs the `nomad` command:

```bash
nomad --version
Installing nomad 0.9.4
100%|************************| 26.8M/26.8M [00:07<00:00, 3.55MiB/s]
Nomad v0.9.4 (a81aa846a45fb8248551b12616287cb57c418cd6)

```
`ozy` will notice that `nomad` is not installed, and will go grab it based on the `~/.ozy/ozy.yaml` configuration file. Note, you can override the version -- for example, if you want to run a later version of `nomad` -- see the [pinning versions](#Pinning-Versions) section below. Additionally you can use `ozy run` to specify a specific version for a one-off execution.

### Getting info on supported commands
```bash
$ ozy info
ozy v0.0.1
Team URL: http://localhost:8000/sample-team-conf.yaml
Team config name: Aquatic Team Configuration
  nomad: nomad 0.9.5 (zip installer from https://releases.hashicorp.com/nomad/0.9.5/nomad_0.9.5_linux_amd64.zip)
  vault: vault 1.2.3 (zip installer from https://releases.hashicorp.com/vault/1.2.3/vault_1.2.3_linux_amd64.zip)
  terraform: terraform 0.12.13 (zip installer from https://releases.hashicorp.com/terraform/0.12.13/terraform_0.12.13_linux_amd64.zip)
```

### Updating
Assuming the config on the remote url has changed to a new version of nomad:
```bash
$ ozy update
100%|************************| 3.86k/3.86k [00:00<00:00, 17.3MiB/s]
Upgrading nomad from 0.9.3 to 0.9.4
Upgrading vault from 1.2.2 to 1.2.3
```

This will cause ozy to re-fetch your team's ozy config url, and note the updates. 
```bash
$ ozy update --dry-run
100%|************************| 3.86k/3.86k [00:00<00:00, 17.3MiB/s]
Would upgrade nomad from 0.9.3 to 0.9.4
Would upgrade vault from 1.2.2 to 1.2.3
Dry run only - no changes made
```

### Eagerly installing all versions
If you're the kind of person who would prefer a fail-fast; or you're going to be working offline for a bit, then you can ensure all the current versions of supported applications are installed:
```bash
$ ozy install-all
Installing nomad 0.9.4
...
```

### Initializing
If you need to completely re-initialize, or are wondering what happens behind the scenes when you first install `ozy`, then this is the place for you. The `ozy init` command takes a URL and uses that to fetch the `~/.ozy/ozy.yaml` for your team. It also remembers that URL (in `~/.ozy/ozy.user.yaml`) so that subsequent [updates](#updating) fetch from there too.
```bash
$ ozy init https://some.ozy.server.net/ozy.yaml
ozy is installed and is ready to run
```

### Pinning Versions
Individual projects can choose to pin versions of apps by creating a `.ozy.yaml` file in the directory that they are in. When you run ozy in that directory, ozy will use the versions listed in that file in preference to the system wide ozy found in `~/.ozy/ozy.yaml`

For example, assuming you are in some directory and `ozy` is in your `PATH`:
```bash
$ echo "apps:
  nomad:
    version: 0.10.0" > .ozy.yaml 
$ nomad --version
Installing nomad 0.10.0
100%|************************| 33.0M/33.0M [00:08<00:00, 3.85MiB/s]
Nomad v0.10.0 (25ee121d951939504376c70bf8d7950c1ddb6a82)
```

Directories are walked up from the current directory, looking for `~/.ozy.yaml` files. Settings are applied from the combination of all configuration files found, with the most deeply-nested yaml file settings taking precedence. 

### Running a command through ozy
You can directly run a command through ozy:

```bash
$ ozy run nomad -- --help
Usage: nomad [-version] [-help] [-autocomplete-(un)install] <command> [args]

Common commands:
...
```

You can also specify a specific version:
```bash
$ ozy run nomad --version 0.9.1 -- --version
Installing nomad 0.9.1
100%|************************| 25.4M/25.4M [00:00<00:00, 73.7MiB/s]
Nomad v0.9.1 (4b2bdbd9ab68a27b10c2ee781cceaaf62e114399)
```


### Removing a command
TODO: this
```bash
$ ozy rm nomad
```
The real nomad lives in `~/.cache/ozy/nomad/0.9.4/nomad`. The symlink in `~/.ozy/bin/nomad` points to ozy. `rm` will remove the symlink, which is as good as deleting the executable.  

### Full cleanup
TODO: this
```bash
$ ozy clean
```
This will remove `~/.cache/ozy`! 

## Writing your own ozy.yaml

TODO

## Making a release

To make a release of ozy:

* Update the `__version__` in `ozy/__init__.py`
* Ensure the `RELEASE_NOTES.md` are updated
* Push the changes to GitHub
* Create a tag of the form `v1.2.3` and push it to GitHub
* GH actions will make the binaries automatically
* Go to https://github.com/aquanauts/ozy/releases and check everything's ok
* Update the release description and make non-draft if all's ok
