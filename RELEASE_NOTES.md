# ozy release notes

## 0.0.6
* Adds support for post installation commands
* New command-line "install" to force install a subset of apps
* Support for OSX

## 0.0.5
* Add support for `pip` installers (via `conda`).

## 0.0.4
* Fix logging crash on `ozy install-all`

## 0.0.3
* Fix for `ozy` running python programs: `PYTHONPATH` and `LD_LIBRARY_PATH` were left hijacked by
  `ozy`'s package system (pyinstaller).

## 0.0.2
* Bug fix in makefile-config


## 0.0.1-pre (First test release)
* Support makefile-config

## 0.0.0 (Work in progress) 
* Init, info, update implemented
* Apps! Support for `nomad`, `terraform`, and `vault` (general support for any Hashicorp thing)
* Working [README.md](README.md) 
* One-line installs a la lake-client 
* Supports installing `conda` packages


---

TODO
* Check the shas and the signed security files
* Support `rm`
* Support `clean`
* Open source! 
* "flock" for multiple ozy invocations at once?

More Apps!
* `docker`
* `sshfs`
