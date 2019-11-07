import logging
import os
from collections import ChainMap

import yaml

from ozy import OzyError
from ozy.files import walk_up_dirs, get_ozy_dir

_LOGGER = logging.getLogger(__name__)


def load_config(overrides=None):
    # Annoyingly can't just use a chainmap here as nested maps don't work the way we'd like
    config = parse_ozy_conf(f"{get_ozy_dir()}/ozy.yaml")
    ozy_files = []
    for path in walk_up_dirs(os.getcwd()):
        conf_file = os.path.join(path, '.ozy.yaml')
        if os.path.isfile(conf_file):
            ozy_files.append(conf_file)
    for path in reversed(ozy_files):
        apply_overrides(parse_ozy_conf(path), config)
    if overrides:
        apply_overrides(overrides, config)
    _LOGGER.debug(config)
    return config


def apply_overrides(source, destination):
    for key, value in source.items():
        if isinstance(value, dict):
            node = destination.setdefault(key, {})
            apply_overrides(value, node)
        else:
            destination[key] = value

    return destination


def resolve(config, templates):
    if 'template' in config:
        template_name = config['template']
        if template_name not in templates:
            raise OzyError(f"Unable to find template '{template_name}'")
        config = ChainMap(config,
                          templates[template_name])  # TODO had these the wrong way round to start with. make a test
    return {key: safe_expand(config, value) for key, value in config.items()}


def safe_expand(format_params, to_expand):
    if isinstance(to_expand, list):
        return [safe_expand(format_params, x) for x in to_expand]
    elif not isinstance(to_expand, str):
        return to_expand
    try:
        return to_expand.format(**format_params)
    except KeyError as ke:
        raise OzyError(f"Could not find key {ke} in expansion '{to_expand}' with params '{format_params}'")


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


def parse_ozy_conf(ozy_file_name):
    _LOGGER.debug("Parsing config %s", ozy_file_name)
    with open(ozy_file_name, "r") as ofnh:
        yaml_content = yaml.load(ofnh, Loader=yaml.UnsafeLoader)
        return yaml_content
