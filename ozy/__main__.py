import logging
import os
import shutil
import sys

import click
import coloredlogs

from ozy import OzyException, find_tool, install_if_needed_and_get_path_to_tool_and_rename_me, download_to, \
    get_ozy_dir, ensure_ozy_dirs, get_ozy_bin_dir, parse_ozy_conf, softlink, save_ozy_user_conf, load_ozy_user_conf, \
    load_config, App

_LOGGER = logging.getLogger(__name__)

PATH_TO_ME = None  # TODO find a better way
IS_SINGLE_FILE = False  # TODO find a better way


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
    """Initialise and install ozy, with configuration from the given URL."""
    ensure_ozy_dirs()
    user_conf = load_ozy_user_conf()
    # TODO: make sure it isn't there already? upgrade/update instead?
    ozy_conf_filename = f"{get_ozy_dir()}/ozy.yaml"
    download_to(ozy_conf_filename, url)
    ozy_bin_dir = get_ozy_bin_dir()
    root_conf = parse_ozy_conf(ozy_conf_filename)  ## TODO think how this interacts with local config files

    symlink_binaries(ozy_bin_dir, root_conf)
    user_conf['url'] = url
    save_ozy_user_conf(user_conf)

    if check_path(ozy_bin_dir):
        _LOGGER.info("ozy is installed, but needs a little more setup work:")
        show_path_warning(ozy_bin_dir)
    else:
        _LOGGER.info("ozy is installed and is ready to run")


def symlink_binaries(ozy_bin_dir, config):
    global PATH_TO_ME, IS_SINGLE_FILE
    if IS_SINGLE_FILE:
        dest_filename = os.path.join(ozy_bin_dir, 'ozy')
        if os.path.samefile(PATH_TO_ME, dest_filename):
            _LOGGER.debug("Not copying anything as we're already in the right place")
        else:
            _LOGGER.debug("Copying single-file ozy distribution")
            if os.path.exists(dest_filename):
                os.unlink(dest_filename)
            shutil.copyfile(PATH_TO_ME, dest_filename)
            shutil.copymode(PATH_TO_ME, dest_filename)
    else:
        _LOGGER.debug("Symlinking dev ozy")
        softlink(from_command=PATH_TO_ME, to_command='ozy', ozy_bin_dir=ozy_bin_dir)
    for app in config['apps']:
        if not softlink(from_command='ozy', to_command=app, ozy_bin_dir=ozy_bin_dir):
            _LOGGER.info("Supporting app '%s'", app)


def show_path_warning(ozy_bin_dir):
    _LOGGER.warning("-" * 80)
    _LOGGER.warning("Please ensure '%s' is on your path", ozy_bin_dir)
    _LOGGER.info("bash shell users:")
    _LOGGER.info("  bash$  echo -e '# ozy support\\nexport PATH=%s:$PATH' >> ~/.bashrc")
    _LOGGER.info("  then restart your shell sessions")
    _LOGGER.info("zsh shell users:")
    _LOGGER.info("  zsh$  # some kind of wizardry here TODO!")
    _LOGGER.info("fish shell users: ")
    _LOGGER.info("  fish$  set --universal fish_user_paths %s $fish_user_paths", ozy_bin_dir)
    _LOGGER.warning("-" * 80)


def check_path(ozy_bin_dir):
    real_paths = set(os.path.realpath(path) for path in os.getenv("PATH").split(":"))
    if os.path.realpath(ozy_bin_dir) in real_paths:
        return True


@main.command()
@click.option("--url", metavar="URL", type=str)
def update(url=None):
    """Update configuration from the remote URL."""
    user_conf = load_ozy_user_conf()
    if not url:
        if 'url' not in user_conf:
            raise OzyException('Missing url in configuration')
        url = user_conf['url']
    ozy_conf_filename = f"{get_ozy_dir()}/ozy.yaml"
    download_to(ozy_conf_filename, url)
    ozy_bin_dir = get_ozy_bin_dir()
    user_conf['url'] = url
    save_ozy_user_conf(user_conf)
    root_conf = parse_ozy_conf(ozy_conf_filename)  ## TODO think how this interacts with local config files
    symlink_binaries(ozy_bin_dir, root_conf)


@main.command()
def info():
    """Print information about the installation and configuration."""
    _LOGGER.info("ozy v0.0.0")  # TODO version
    ozy_bin_dir = get_ozy_bin_dir()
    if not check_path(ozy_bin_dir):
        show_path_warning(ozy_bin_dir)
    user_config = load_ozy_user_conf()
    _LOGGER.info("Team URL: %s", user_config.get("url", "(unset)"))
    config = load_config()
    _LOGGER.info("Team config name: %s", config.get("name", "(unset)"))
    for app_name in config['apps']:
        app = App(app_name, config)
        _LOGGER.info("  %s: %s", app_name, app)


@main.command()
def install_all():
    """Ensures all applications are installed at their current prevailing versions."""
    config = load_config()
    for app_name in config['apps']:
        app = App(app_name, config)
        app.ensure_installed()


@main.command()
def sync():
    """
    Synchronise any local changes.

    If you're defining new applications in local user files, you can use this to ensure
    the relevant symlinks are created in your ozy bin directory.
    """
    symlink_binaries(get_ozy_bin_dir(), load_config())


def app_main(path_to_ozy, argv0, arguments, is_single_file):
    global PATH_TO_ME, IS_SINGLE_FILE
    PATH_TO_ME = os.path.realpath(path_to_ozy)
    IS_SINGLE_FILE = is_single_file

    invoked_as = os.path.basename(argv0)
    if invoked_as in ('ozy', '__main__.py'):
        main(prog_name='ozy', args=arguments)
    else:
        coloredlogs.install(fmt='%(message)s', level='INFO')
        tool = find_tool(invoked_as)
        if not tool:
            raise OzyException(f"TODO better, couldn't find {invoked_as}")
        path = install_if_needed_and_get_path_to_tool_and_rename_me(tool)
        try:
            os.execv(path, [path] + arguments)
        except Exception as e:
            _LOGGER.error("Unable to execute %s: %s", path, e)
            raise


if __name__ == "__main__":
    app_main(sys.argv[1], sys.argv[1], sys.argv[2:], False)
