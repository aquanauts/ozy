# ozy

`ozy` is a native Python program that makes it easy for you and your team to share and colloborate on commonly used programs such as Hashicorp's `vault`, `python`, etc. 

## Getting Started
First, install ozy: follow the instructions on https://ozy.aq.tc/. This will cause `ozy` to be installed in `~/.ozy`. In this directory there are some `ozy.*.yaml` files, and a `bin` directory. As part of the installation `ozy` will check and tell you how to put `~/.ozy/bin` into your path. It will place magic symlinks there that let it masquerade as all the applications it can install.

To see what the installer is doing see [ozy init](#Initializing) - but in short it is configuring itself from an Aquatic-specific endpoint.

## Running a command
Assume that your installation has been configured to support Hashicorp's [nomad](https://www.nomadproject.io/). One simply runs the `nomad` command:

```bash
nomad --version
Installing nomad 0.9.4
100%|█████████████| 26.8M/26.8M [00:07<00:00, 3.55MiB/s]
Nomad v0.9.4 (a81aa846a45fb8248551b12616287cb57c418cd6)

```
`ozy` will notice that `nomad` is not installed, and will go grab it based on the `~/.ozy/ozy.yaml` configuration file. Note, you can override in install -- for example, if you want to run a later version of `nomad` -- see the [pinning versions](#Pinning-Versions) section below.

##  Getting info on supported commands
```bash
$ ozy info
ozy v0.0.0
Team URL: http://localhost:8000/sample-team-conf.yaml
Team config name: Aquatic Team Configuration
  nomad: nomad 0.9.5 (zip installer from https://releases.hashicorp.com/nomad/0.9.5/nomad_0.9.5_linux_amd64.zip)
  vault: vault 1.2.3 (zip installer from https://releases.hashicorp.com/vault/1.2.3/vault_1.2.3_linux_amd64.zip)
  terraform: terraform 0.12.13 (zip installer from https://releases.hashicorp.com/terraform/0.12.13/terraform_0.12.13_linux_amd64.zip)
```

## Updating
Assuming the config on the remote url has changed to a new version of nomad:
TODO: this isn't the output currently
```bash
$ ozy update
team version of nomad changed from 0.9.4 to 0.9.5
team version of vault changed from 1.2.3 to 1.2.5
run the app to download and install 
```

This will cause ozy to re-fetch your team's ozy config url, and note the updates. 
TODO: dry-run not supported yet
```bash
$ ozy update --dry-run 
team version of nomad would change from 0.9.4 to 0.9.5
team version of vault would change from 1.2.3 to 1.2.5
```

## Eagerly installing all versions
If you're the kind of person who would prefer a fail-fast; or you're going to be working offline for a bit, then you can ensure all the current versions of supported applications are installed:
```bash
$ ozy install-all
Installing nomad 0.9.4
...
```

## Initializing
If you need to completely re-initialize, or are wondering what happens behind the scenes when you first install `ozy`, then this is the place for you. The `ozy init` command takes a URL and uses that to fetch the `~/.ozy/ozy.yaml` for your team. It also remembers that URL (in `~/.ozy/ozy.user.yaml`) so that subsequent [updates](#updating) fetch from there too.
```bash
$ ozy init https://some.ozy.server.net/ozy.yaml
ozy is installed and is ready to run
```

## Pinning Versions
Individual projects can choose to pin versions of apps by creating a `.ozy.yaml` file in the directory that they are in. When you run ozy in that directory, ozy will use the versions listed in that file in preference to the system wide ozy found in `~/.ozy/ozy.yaml`

For example, assuming you are in some directory and `ozy` is in your `PATH`:
```bash
$ echo "apps:
  nomad:
    version: 0.10.0" > .ozy.yaml 
$ nomad --version
Installing nomad 0.10.0
100%|█████████████| 33.0M/33.0M [00:08<00:00, 3.85MiB/s]
Nomad v0.10.0 (25ee121d951939504376c70bf8d7950c1ddb6a82)
```

Directories are walked up from the current directory, looking for `~/.ozy.yaml` files. Settings are applied from the combination of all configuration files found, with the most deeply-nested yaml file settings taking precedence. 

## Running a command through ozy
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
100%|█████████████| 25.4M/25.4M [00:00<00:00, 73.7MiB/s]
Nomad v0.9.1 (4b2bdbd9ab68a27b10c2ee781cceaaf62e114399)
```


## Removing a command
TODO: this
```bash
$ ozy rm nomad
```
The real nomad lives in `~/.cache/ozy/nomad/0.9.4/nomad`. The symlink in `~/.ozy/bin/nomad` points to ozy. `rm` will remove the symlink, which is as good as deleting the executable.  

## Full cleanup
TODO: this
```bash
$ ozy clean
```
This will remove `~/.cache/ozy`! 

---

### Matt's original write-up
See [this Google doc](https://docs.google.com/document/d/1CkUMCaoJg0g5A60B5nxkKAGQ5Tfhm_WxrfUJEgBUjpU/edit#)
