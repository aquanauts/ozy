# ozy

`ozy` is a native Python program that makes it easy for you and your team to share and colloborate on commonly used programs such as Hashicorp's `vault`, `python`, etc. 

## Getting Started
First, [download ozy](convenient download link here). This will cause `ozy` to install itself details details mking `~/.ozy`, installing itself and the `.ozyconf` file there, and adding that directory to `$PATH`. 

Next, you need to initialize `ozy` with a url which contains your team's blessed set of apps:

```bash
$ ozy init https://github.com/myteam/ozy/.ozyconf
Supporting app 'nomad'
Supporting app 'vault'
Supporting app 'terraform'
``` 

This will:
1) Create `$HOME/.ozy`
2) Cache the remote team yaml file as `$HOME/.ozy/ozy.yaml` 
3) Simlink `ozy` into `/home/$HOME/.ozy/bin/ozy`
4) Creates a `/home/.ozy/ozy.user.yaml` file that stores the url used to init ozy.

## Running a command
Assume that your `ozy.yaml` has Hashicorp's [nomad](nomad url) in it.

```bash
nomad --version
Installing nomad 0.9.4
100%|█████████████ 26.8M/26.8M [00:07<00:00, 3.55MiB/s]
Nomad v0.9.4 (a81aa846a45fb8248551b12616287cb57c418cd6)

```
`ozy` will notice that `nomad` is not installed, and will go grab it based on the `ozy.yaml` file. Note, you can override in install -- for example, if you want to run a later version of `nomad` -- see the [pinning versions](Pinning-Versions) section below.

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
Assuming the config on the remote url has changed to a new version of nomad, 
```bash
$ ozy update
team version of nomad changed from 0.9.4 to 0.9.5
team version of vault changed from 1.2.3 to 1.2.5
run the app to download and install 
```
This will cause ozy to re-fetch your team's ozy config url, and note the updates. 
```bash
$ ozy update --dry-run 
team version of nomad would change from 0.9.4 to 0.9.5
team version of vault would change from 1.2.3 to 1.2.5
```

TODO: make the update happen by default! 

## Pinning Versions
Individual projects can choose to pin versions of apps by creating a `.ozy.yaml` file in the directory that they are in. When you run ozy in that directory, ozy will use the versions listed in that file in preference to the system wide ozy found in `~/.ozy/ozy.yaml`

For example, assuming you are in some directory and `ozy` is in your `PATH`:
```bash
$ echo "apps:
  nomad:
    version: 0.10.0" > .ozy.yaml 
$ nomad --version
Installing nomad 0.10.0
100%|█████████████████████████████████████| 33.0M/33.0M [00:08<00:00, 3.85MiB/s]
Nomad v0.10.0 (25ee121d951939504376c70bf8d7950c1ddb6a82)

```


## Running a command through ozy
```bash
$ ozy run nomad 0.9.4 
```

## Removing a command
```bash
$ ozy rm nomad
```
The real nomad lives in `~/.cache/ozy/nomad/0.9.4/nomad`. The simlink in `~/.ozy/bin/nomad` points to ozy. `rm` will remove the simlink, which is as good as deleting the executable.  

## Full cleanup
```bash
$ ozy clean
```
This will remove `~/.cache/ozy`! 



### Matt's original write-up
See [this Google doc](https://docs.google.com/document/d/1CkUMCaoJg0g5A60B5nxkKAGQ5Tfhm_WxrfUJEgBUjpU/edit#)

