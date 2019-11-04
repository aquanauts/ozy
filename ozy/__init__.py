import logging
import os
import shutil
import tarfile
from collections import ChainMap
from subprocess import check_call
from tempfile import NamedTemporaryFile
from typing import BinaryIO
from zipfile import ZipFile

import requests
import yaml
from tqdm import tqdm

_LOGGER = logging.getLogger(__name__)

DOWNLOAD_CHUNK_SIZE = 128 * 1024


class OzyException(Exception):
    pass


def safe_expand(format_params, to_expand):
    if isinstance(to_expand, list):
        return [safe_expand(format_params, x) for x in to_expand]
    elif not isinstance(to_expand, str):
        return to_expand
    try:
        return to_expand.format(**format_params)
    except KeyError as ke:
        raise OzyException(f"Could not find key {ke} in expansion '{to_expand}' with params '{format_params}'")


def resolve(config, templates):
    if 'template' in config:
        template_name = config['template']
        if template_name not in templates:
            raise OzyException(f"Unable to find template '{template_name}'")
        config = ChainMap(config,
                          templates[template_name])  # TODO had these the wrong way round to start with. make a test
    return {key: safe_expand(config, value) for key, value in config.items()}


def ensure_keys(name, config, *keys):
    for required_key in keys:
        if required_key not in config:
            raise OzyException(f"Missing required key '{required_key}' in '{name}'")


class Installer:
    def __init__(self, name, config, *required_keys, **default_keys):
        self._name = name
        self._config = default_keys.copy()
        for required_key in required_keys:
            if required_key not in config:
                raise OzyException(f"Missing required key '{required_key}' in '{name}'")
            self._config[required_key] = config[required_key]
        for optional_key in default_keys.keys():
            if optional_key in config:
                self._config[optional_key] = config[optional_key]

    def config(self, name):
        return self._config[name]

    def install(self, to_dir):
        raise RuntimeError("Must be overridden")


# TODO support sha256, sha256_signature and sha256_gpg_key
# TODO tests for installers!
class SingleBinaryZipInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url', app_name=name)

    def __str__(self):
        return f'zip installer from {self.config("url")}'

    def install(self, to_dir):
        app_name = self.config('app_name')
        url = self.config('url')
        os.makedirs(to_dir)
        app_path = os.path.join(to_dir, app_name)
        with NamedTemporaryFile() as temp_file:
            download_to_file_obj(temp_file, url)
            temp_file.flush()
            zf = ZipFile(temp_file.name)
            contents = zf.namelist()
            if len(contents) != 1:
                raise OzyException(f"More than one file in the zipfile at {url}! ({contents})")
            with open(app_path, 'wb') as out_file:
                with zf.open(contents[0]) as in_file:
                    out_file.write(in_file.read())
            os.chmod(app_path, 0o774)


class TarballInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url')

    def __str__(self):
        return f'tar installer from {self.config("url")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        url = self.config('url')
        with NamedTemporaryFile() as temp_file:
            download_to_file_obj(temp_file, url)
            temp_file.flush()
            tf = tarfile.open(temp_file.name, 'r')
            tf.extractall(to_dir)


class SingleFileInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url', app_name=name)

    def __str__(self):
        return f'file installer from {self.config("url")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        url = self.config('url')
        app_path = os.path.join(to_dir, self.config('app_name'))
        with open(app_path, 'wb') as output_file:
            download_to_file_obj(output_file, url)
        os.chmod(app_path, 0o774)


class ShellInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url', 'shell_args')

    def __str__(self):
        return f'shell file installer from {self.config("url")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        url = self.config('url')
        with NamedTemporaryFile(delete=False) as temp_file:
            download_to_file_obj(temp_file, url)
            temp_file.close()
            os.chmod(temp_file.name, 0o774)
            env = os.environ.copy()
            env['INSTALL_DIR'] = to_dir
            check_call([temp_file.name] + self.config("shell_args"), env=env)
            os.unlink(temp_file.name)


SUPPORTED_INSTALLERS = dict(
    single_binary_zip=SingleBinaryZipInstaller,
    tarball=TarballInstaller,
    single_file=SingleFileInstaller,
    shell_install=ShellInstaller
)


class App:
    def __init__(self, name, root_config):
        self._name = name
        self._root_config = root_config
        self._config = resolve(root_config['apps'][name], self._root_config.get('templates', {}))
        self._executable_path = self._config.get('executable_path', self.name)
        self._relocatable = self._config.get('relocatable', True)
        ensure_keys(name, self._config, 'version', 'type')
        install_type = self._config['type']
        if install_type not in SUPPORTED_INSTALLERS:
            raise OzyException(f"Unsupported installation type '{install_type}'")
        self._installer = SUPPORTED_INSTALLERS[install_type](name, self._config)

    def __str__(self):
        return f'{self.name} {self._config["version"]} ({self._installer})'

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
    def install_path(self) -> str:
        return os.path.join(get_ozy_cache_dir(), self.name, self.version)

    @property
    def executable(self) -> str:
        return os.path.join(self.install_path, self._executable_path)

    def is_installed(self) -> bool:
        return os.path.isdir(self.install_path)

    def install(self):
        _LOGGER.info("Installing %s %s", self.name, self.version)
        if not self._relocatable:
            _LOGGER.debug("Installing directly to final path")
            if os.path.exists(self.install_path):
                shutil.rmtree(self.install_path)
            try:
                self._installer.install(self.install_path)
            except Exception:
                shutil.rmtree(self.install_path)
                raise
        else:
            temp_install_dir = self.install_path + ".tmp"
            if os.path.exists(temp_install_dir):
                shutil.rmtree(temp_install_dir)
            try:
                self._installer.install(temp_install_dir)
                if os.path.exists(self.install_path):
                    shutil.rmtree(self.install_path)
                os.rename(temp_install_dir, self.install_path)
            except Exception:
                shutil.rmtree(temp_install_dir)
                raise

    def ensure_installed(self):
        if not self.is_installed():
            self.install()


def install_if_needed_and_get_path_to_tool_and_rename_me(app):
    app.ensure_installed()
    return app.executable


def walk_up_dirs(path):
    path = os.path.realpath(path)
    previous_path = None
    while path != previous_path:
        yield path
        previous_path = path
        path = os.path.realpath(os.path.join(path, '..'))


def apply_overrides(source, destination):
    for key, value in source.items():
        if isinstance(value, dict):
            node = destination.setdefault(key, {})
            apply_overrides(value, node)
        else:
            destination[key] = value

    return destination


def load_config():
    # Annoyingly can't just use a chainmap here as nested maps don't work the way we'd like
    config = parse_ozy_conf(f"{get_ozy_dir()}/ozy.yaml")
    ozy_files = []
    for path in walk_up_dirs(os.getcwd()):
        conf_file = os.path.join(path, '.ozy.yaml')
        if os.path.isfile(conf_file):
            ozy_files.append(conf_file)
    for path in reversed(ozy_files):
        apply_overrides(parse_ozy_conf(path), config)
    _LOGGER.debug(config)
    return config


def find_tool(tool):
    config = load_config()
    if tool in config['apps']:
        return App(tool, config)
    else:
        return None


def download_to(dest_file_name: str, url: str):
    _LOGGER.debug("Downloading %s to %s", url, dest_file_name)
    dest_file_temp = dest_file_name + ".tmp"
    try:
        with open(dest_file_temp, 'wb') as dest_file_obj:
            download_to_file_obj(dest_file_obj, url)
        os.rename(dest_file_temp, dest_file_name)
    except Exception:
        os.unlink(dest_file_temp)
        raise


def download_to_file_obj(dest_file_obj: BinaryIO, url: str):
    response = requests.get(url, stream=True)
    if not response.ok:
        raise OzyException(f"Unable to fetch url '{url}' - {response}")
    total_size = int(response.headers.get('content-length', 0))
    with tqdm(total=total_size, unit='iB', unit_scale=True) as t:
        for data in response.iter_content(DOWNLOAD_CHUNK_SIZE):
            t.update(len(data))
            dest_file_obj.write(data)


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
    _LOGGER.debug("Parsing config %s", ozy_file_name)
    with open(ozy_file_name, "r") as ofnh:
        yaml_content = yaml.load(ofnh, Loader=yaml.UnsafeLoader)
        return yaml_content


def softlink(from_command, to_command, ozy_bin_dir):
    # assume this linkage will ONLY be called by ozy
    path_to_app = os.path.join(ozy_bin_dir, to_command)
    was_there = os.path.exists(path_to_app)
    if was_there:
        _LOGGER.debug(f"Clobbering symlink path {path_to_app}")
        os.unlink(path_to_app)
    os.symlink(from_command, path_to_app)
    return was_there


def load_ozy_user_conf():
    user_conf_file = get_user_conf_file()
    user_conf = dict()
    if os.path.exists(user_conf_file):
        user_conf = parse_ozy_conf(user_conf_file)
    return user_conf


def save_ozy_user_conf(config):
    with open(get_user_conf_file(), 'w') as user_conf_file_obj:
        yaml.dump(config, user_conf_file_obj)


def get_user_conf_file():
    user_conf_file = f"{get_ozy_dir()}/ozy.user.yaml"
    return user_conf_file
