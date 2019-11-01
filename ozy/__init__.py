
class OzyException(Exception):
    pass


def safe_expand(format_params, to_expand):
    try:
        return to_expand.format(**format_params)
    except KeyError as ke:
        raise OzyException(f"Could not find key {ke} in expansion '{to_expand}' with params '{format_params}'")

class Tool:
    def __init__(self, name, config):
        self._name = name
        self._config = config

    @property
    def name(self):
        return self._name

    @property
    def config(self):
        return self._config


def install_if_needed_and_get_path_to_tool(tool):
    return tool.config['path']


def find_tool(tool):
    if tool == 'test_nomad':
        return Tool('test_nomad', dict(path="/bin/ls"))
    return None

