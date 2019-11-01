import logging
import os

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
    os.makedirs(get_ozy_dir(), exist_ok=True)
    os.makedirs(get_ozy_bin_dir(), exist_ok=True)


def get_ozy_dir() -> str:
    if 'HOME' in os.environ:
        return f"{os.environ['HOME']}/.ozy"
    else:
        raise OzyException("HOME env variable not found")


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