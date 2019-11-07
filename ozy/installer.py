from ozy import OzyError


class Installer:
    def __init__(self, name, config, *required_keys, **default_keys):
        self._name = name
        self._config = default_keys.copy()
        for required_key in required_keys:
            if required_key not in config:
                raise OzyError(f"Missing required key '{required_key}' in '{name}'")
            self._config[required_key] = config[required_key]
        for optional_key in default_keys.keys():
            if optional_key in config:
                self._config[optional_key] = config[optional_key]

    def config(self, name):
        return self._config[name]

    def install(self, to_dir):
        raise RuntimeError("Must be overridden")

# TODO tests for installers!
