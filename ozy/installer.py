from typing import Any

from ozy import OzyError


class Installer:
    def __init__(self, name: str, config: dict, *required_keys, **default_keys):
        self._name = name
        self._config = default_keys.copy()
        self._executable_path = config.get('executable_path', name)

        for required_key in required_keys:
            if required_key not in config:
                raise OzyError(f"Missing required key '{required_key}' in '{name}'")
            self._config[required_key] = config[required_key]
        for optional_key in default_keys.keys():
            if optional_key in config:
                self._config[optional_key] = config[optional_key]

    def config(self, name) -> Any:
        return self._config[name]

    def install(self, to_dir: str):
        raise RuntimeError("Must be overridden")

# TODO tests for installers!
