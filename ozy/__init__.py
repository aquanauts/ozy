import logging
import os
from collections import ChainMap

import requests
import yaml
from tqdm import tqdm

_LOGGER = logging.getLogger(__name__)

DOWNLOAD_CHUNK_SIZE = 128 * 1024


class OzyException(Exception):
    pass


def safe_expand(format_params, to_expand):
    try:
        return to_expand.format(**format_params)
    except KeyError as ke:
        raise OzyException(f"Could not find key {ke} in expansion '{to_expand}' with params '{format_params}'")


def resolve(config, templates):
    if 'template' in config:
        template_name = config['template']
        if template_name not in templates:
            raise OzyException(f"Unable to find template '{template_name}'")
        config = ChainMap(config, templates[template_name])
    return {key: safe_expand(config, value) for key, value in config.items()}


class App:
    def __init__(self, name, root_config):
        self._name = name
        self._root_config = root_config
        self._config = resolve(root_config['apps'][name], self._root_config.get('templates', {}))
        for required_key in ('version', 'type'):
            if required_key not in self._config:
                raise OzyException(f"Missing required key '{required_key}' in '{name}'")
        print(self._config)
        stop

    @property
    def name(self) -> str:
        return self._name

    @property
    def config(self) -> dict:
        return self._config

    @property
    def version(self) -> str:
        return self._config['version']

    @property
    def type(self) -> str:
        return self._config['type']

    @property
    def install_path(self) -> str:
        return os.path.join(get_ozy_cache_dir(), self.name, self.version)

    @property
    def executable(self) -> str:
        return os.path.join(self.install_path, self.name)  # TODO needs to be configurable

    def is_installed(self) -> bool:
        return os.path.isdir(self.install_path)

    def install(self):
        if self.install_type != 'single_binary':
            raise RuntimeError(f'Unsupported installer type {self.install_type}')
        pass

    def ensure_installed(self):
        if not self.is_installed():
            self.install()


def install_if_needed_and_get_path_to_tool_and_rename_me(app):
    app.ensure_installed()
    return app.executable


def load_config():
    return parse_ozy_conf(f"{get_ozy_dir()}/ozy.conf.yaml")


def find_tool(tool):
    config = load_config()
    if tool in config['apps']:
        return App(tool, config)
    else:
        return None


def download_to(dest_file_name: str, url: str):
    _LOGGER.debug("Downloading %s to %s", url, dest_file_name)
    response = requests.get(url, stream=True)
    if not response.ok:
        raise OzyException(f"Unable to fetch url '{url}' - {response}")
    total_size = int(response.headers.get('content-length', 0))
    dest_file_temp = dest_file_name + ".tmp"
    try:
        with open(dest_file_temp, 'wb') as dest_file_obj:
            with tqdm(total=total_size, unit='iB', unit_scale=True) as t:
                for data in response.iter_content(DOWNLOAD_CHUNK_SIZE):
                    t.update(len(data))
                    dest_file_obj.write(data)
        os.rename(dest_file_temp, dest_file_name)
    except Exception:
        os.unlink(dest_file_temp)
        raise


def ensure_ozy_dirs():
    [os.makedirs(p, exist_ok=True) for p in (get_ozy_dir(), get_ozy_bin_dir(), get_ozy_cache_dir())]


def get_home_dir() -> str:
    if 'HOME' in os.environ:
        return os.environ['HOME']
    raise OzyException("HOME env variable not found")


def get_ozy_dir() -> str:
    return f"{get_home_dir()}/.ozy"


def get_ozy_cache_dir() -> str:
    return os.path.join(os.getenv('XDG_CACHE_HOME', f"{get_home_dir()}/.cache"), 'ozy')


def get_ozy_bin_dir() -> str:
    return os.path.join(get_ozy_dir(), "bin")


def parse_ozy_conf(ozy_file_name):
    with open(ozy_file_name, "r") as ofnh:
        yaml_content = yaml.load(ofnh, Loader=yaml.UnsafeLoader)
        return yaml_content


def softlink(from_command, to_command, ozy_bin_dir, ):
    # assume this linkage will ONLY be called by ozy
    path_to_app = os.path.join(ozy_bin_dir, to_command)
    if os.path.exists(path_to_app):
        _LOGGER.debug(f"Clobbering symlink path {path_to_app}")
        os.unlink(path_to_app)
    os.symlink(from_command, path_to_app)
