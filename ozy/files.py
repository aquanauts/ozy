import logging
import os

from ozy import OzyError

_LOGGER = logging.getLogger(__name__)


def walk_up_dirs(path):
    path = os.path.realpath(path)
    previous_path = None
    while path != previous_path:
        yield path
        previous_path = path
        path = os.path.realpath(os.path.join(path, '..'))


def ensure_ozy_dirs():
    [os.makedirs(p, exist_ok=True) for p in (get_ozy_dir(), get_ozy_bin_dir(), get_ozy_cache_dir())]


def get_home_dir() -> str:
    if 'HOME' in os.environ:
        return os.environ['HOME']
    raise OzyError("HOME env variable not found")


def get_ozy_dir() -> str:
    return f"{get_home_dir()}/.ozy"


def get_ozy_cache_dir() -> str:
    return os.path.join(os.getenv('XDG_CACHE_HOME', f"{get_home_dir()}/.cache"), 'ozy')


def get_ozy_bin_dir() -> str:
    return os.path.join(get_ozy_dir(), "bin")


def softlink(from_command, to_command, ozy_bin_dir):
    # assume this linkage will ONLY be called by ozy
    path_to_app = os.path.join(ozy_bin_dir, to_command)
    was_there = os.path.exists(path_to_app)
    if was_there:
        _LOGGER.debug(f"Clobbering symlink path {path_to_app}")
        os.unlink(path_to_app)
    os.symlink(from_command, path_to_app)
    return was_there
