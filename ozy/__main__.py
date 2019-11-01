import os
import sys

import click
import coloredlogs
import logging

from ozy import OzyException, find_tool, install_if_needed_and_get_path_to_tool_and_rename_me, download_to, \
    get_ozy_dir, ensure_ozy_dirs, get_ozy_bin_dir, parse_ozy_conf, softlink, save_ozy_user_conf, load_ozy_user_conf

_LOGGER = logging.getLogger(__name__)

INVOKED_AS = sys.argv[1]  # TODO ... make more @clicky


@click.group()
@click.option("--debug/--no-debug", default=False)
def main(debug):
    # TODO detect if redirected and don't do this, etc
    coloredlogs.install(
        fmt='%(message)s',
        level=debug and 'DEBUG' or 'INFO')


@main.command()
@click.argument("url", metavar="URL", type=str)
def init(url):
    ensure_ozy_dirs()
    user_conf = load_ozy_user_conf()
    # TODO: make sure it isn't there already? upgrade/update instead?
    ozy_conf_filename = f"{get_ozy_dir()}/ozy.conf.yaml"
    download_to(ozy_conf_filename, url)
    ozy_bin_dir = os.path.realpath(get_ozy_bin_dir())
    check_path(ozy_bin_dir)
    root_conf = parse_ozy_conf(ozy_conf_filename)  ## TODO think how this interacts with local config files

    # TODO: Copy ourselves there? <tricky>
    symlink_binaries(ozy_bin_dir, root_conf)
    # TODO awesome congratulatory text here

    user_conf['url'] = url
    save_ozy_user_conf(user_conf)


def symlink_binaries(ozy_bin_dir, config):
    path_to_ozy = os.path.realpath(INVOKED_AS)
    softlink(from_command=path_to_ozy, to_command='ozy', ozy_bin_dir=ozy_bin_dir)
    for app in config['apps']:
        if not softlink(from_command=path_to_ozy, to_command=app, ozy_bin_dir=ozy_bin_dir):
            _LOGGER.info("Supporting app '%s'", app)


def check_path(ozy_bin_dir):
    real_paths = set(os.path.realpath(path) for path in os.getenv("PATH").split(":"))
    if ozy_bin_dir not in real_paths:
        _LOGGER.warning("Please place '%s' on your path", ozy_bin_dir)
        # TODO make this nicer! instructions, helpers, etc
    return ozy_bin_dir


@main.command()
@click.option("--url", metavar="URL", type=str)
def update(url=None):
    user_conf = load_ozy_user_conf()
    if not url:
        if 'url' not in user_conf:
            raise OzyException('Missing url in configuration')
        url = user_conf['url']
    ozy_conf_filename = f"{get_ozy_dir()}/ozy.conf.yaml"
    download_to(ozy_conf_filename, url)
    ozy_bin_dir = os.path.realpath(get_ozy_bin_dir())
    user_conf['url'] = url
    save_ozy_user_conf(user_conf)
    root_conf = parse_ozy_conf(ozy_conf_filename)  ## TODO think how this interacts with local config files
    symlink_binaries(ozy_bin_dir, root_conf)


if __name__ == "__main__":
    invoked_as = os.path.basename(sys.argv[1])
    sys.argv = [sys.argv[0]] + sys.argv[2:]
    if invoked_as in ('ozy', '__main__.py'):
        main(prog_name='ozy')
    else:
        coloredlogs.install(fmt='%(message)s', level='INFO')
        tool = find_tool(invoked_as)
        if not tool:
            raise OzyException(f"TODO better, couldn't find {invoked_as}")
        path = install_if_needed_and_get_path_to_tool_and_rename_me(tool)
        os.execv(path, [path] + sys.argv[1:])
