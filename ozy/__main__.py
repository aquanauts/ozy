import logging
import os
import shutil
import sys

import click
import coloredlogs

from ozy import OzyError, __version__
from ozy.app import App, find_app
from ozy.config import load_ozy_user_conf, save_ozy_user_conf, parse_ozy_conf, load_config
from ozy.files import ensure_ozy_dirs, get_ozy_bin_dir, softlink, get_ozy_dir
from ozy.utils import download_to, restore_overridden_env_vars

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

    if not check_path(ozy_bin_dir):
        _LOGGER.info("ozy is installed, but needs a little more setup work:")
        show_path_warning(ozy_bin_dir)
    else:
        _LOGGER.info("ozy is installed and is ready to run")


def symlink_binaries(ozy_bin_dir, config):
    global PATH_TO_ME, IS_SINGLE_FILE
    if IS_SINGLE_FILE:
        dest_filename = os.path.join(ozy_bin_dir, 'ozy')
        if os.path.exists(dest_filename) and os.path.samefile(PATH_TO_ME, dest_filename):
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
    _LOGGER.info("  bash$ echo -e '# ozy support\\nexport PATH=%s:$PATH' >> ~/.bashrc", ozy_bin_dir)
    _LOGGER.info("  then restart your shell sessions")
    _LOGGER.info("zsh shell users:")
    _LOGGER.info(f"  zsh$ # path+=({ozy_bin_dir})\\nexport PATH")
    _LOGGER.info("fish shell users: ")
    _LOGGER.info("  fish$ set --universal fish_user_paths %s $fish_user_paths", ozy_bin_dir)
    _LOGGER.warning("-" * 80)


def check_path(ozy_bin_dir):
    real_paths = set(os.path.realpath(path) for path in os.getenv("PATH").split(":"))
    if os.path.realpath(ozy_bin_dir) in real_paths:
        return True
    return False


@main.command()
@click.option("--url", metavar="URL", type=str, help="configuration URL (default will use previously set)")
@click.option("--dry-run/--no-dry-run", help="make no changes, just show what would happen", default=False)
def update(dry_run, url):
    """Update base configuration from the remote URL."""
    user_conf = load_ozy_user_conf()
    if not url:
        if 'url' not in user_conf:
            raise OzyError('Missing url in configuration')
        url = user_conf['url']
    ozy_conf_filename = f"{get_ozy_dir()}/ozy.yaml"
    tmp_filename = ozy_conf_filename + ".tmp"
    download_to(tmp_filename, url)
    new_conf_root = parse_ozy_conf(tmp_filename)
    old_conf_root = parse_ozy_conf(ozy_conf_filename)

    changed = False
    for app, new_conf in new_conf_root['apps'].items():
        old_conf = old_conf_root['apps'].get(app, None)
        if not old_conf:
            _LOGGER.info('%s new app %s (%s)', "Would install" if dry_run else "Installing", app, new_conf['version'])
            changed = True
        elif old_conf['version'] != new_conf['version']:
            _LOGGER.info('%s %s from %s to %s', "Would upgrade" if dry_run else "Upgrading", app, old_conf['version'],
                         new_conf['version'])
            changed = True

    if not dry_run:
        ozy_bin_dir = get_ozy_bin_dir()
        user_conf['url'] = url
        save_ozy_user_conf(user_conf)
        os.rename(tmp_filename, ozy_conf_filename)
        symlink_binaries(ozy_bin_dir, new_conf_root)
        if not changed:
            _LOGGER.info("No changes made")
    else:
        if changed:
            _LOGGER.info("Dry run only - no changes made")
        else:
            _LOGGER.info("Dry run only - no changes would be made, even without --dry-run")
        os.unlink(tmp_filename)


@main.command()
def info():
    """Print information about the installation and configuration."""
    _LOGGER.info(f"ozy v{__version__}")
    ozy_bin_dir = get_ozy_bin_dir()
    path_ok = check_path(ozy_bin_dir)
    if not path_ok:
        show_path_warning(ozy_bin_dir)
    user_config = load_ozy_user_conf()
    _LOGGER.info("Team URL: %s", user_config.get("url", "(unset)"))
    config = load_config()
    _LOGGER.info("Team config name: %s", config.get("name", "(unset)"))
    for app_name in config['apps']:
        app = App(app_name, config)
        _LOGGER.info("  %s: %s", app_name, app)
        if path_ok:
            found_app = shutil.which(app_name)
            if not found_app:
                _LOGGER.warning("  %s not found on path: perhaps an 'ozy sync' is needed?", app_name)
            else:
                if os.path.realpath(found_app) != os.path.realpath(os.path.join(ozy_bin_dir, app_name)):
                    _LOGGER.warning("  %s is not under ozy control! It was found on your PATH earlier than ozy at %s",
                                    app_name, found_app)


@main.command()
def install_all():
    """Ensures all applications are installed at their current prevailing versions."""
    config = load_config()
    for app_name in config['apps']:
        app = App(app_name, config)
        app.ensure_installed()


@main.command()
@click.argument("apps", metavar="APP...", nargs=-1, required=True, type=str)
def install(apps):
    """Ensures the named applications are installed at their current prevailing versions."""
    config = load_config()
    for app_name in apps:
        if app_name not in config['apps']:
            raise OzyError(f"App '{app_name}' was not found")
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


def _makefile_error(error):
    print(f'$(error "{error}")')  # todo escape
    sys.exit(1)


@main.command()
@click.option("--all-apps/--no-all-apps", help="include all APPs", default=False)
@click.argument("makefile_var", metavar="VAR", type=str)
@click.argument("required_apps", metavar="APP", nargs=-1, type=str)
def makefile_config(makefile_var, required_apps, all_apps):
    """
    Checks apps, and prints a single-line Makefile variable.

    Use as an argument to $(eval). Errors will are output as $(error) directives
    to report in make.
    The given variable is defined to be the ozy binary directory, so any app will be
    $(VAR)/app_name. If undefined, you know ozy isn't installed.

    Example:

    \b
    $ cat Makefile
    $(eval $(shell ozy makefile-config OZY_BIN_DIR terraform))
    ifndef OZY_BIN_DIR
    $(error please install ozy)
    endif

    \b
    install:
        $(OZY_BIN_DIR)/terraform apply
    """
    config = load_config()
    ozy_bin_dir = get_ozy_bin_dir()
    path_ok = check_path(ozy_bin_dir)
    if not path_ok:
        _makefile_error("ozy is not on the path")
    if all_apps:
        required_apps = config['apps'].keys()
    if not required_apps:
        _makefile_error("no ozy apps found to configure")
    for app_name in required_apps:
        if app_name not in config['apps']:
            _makefile_error(f"Missing ozy app '{app_name}'")
        app = App(app_name, config)
        found_app = shutil.which(app_name)
        app_in_bin = os.path.join(ozy_bin_dir, app_name)
        if os.path.realpath(found_app) != os.path.realpath(app_in_bin):
            _makefile_error(f"{found_app} found in PATH earlier than ozy: "
                            f"results could be inconsistent (found at {found_app})")
    print(f"{makefile_var}:={ozy_bin_dir}")


@main.command()
@click.option("--version", metavar="VERSION", type=str)
@click.argument("app", metavar="APP", type=str)
@click.argument("arguments", metavar="ARG", nargs=-1, type=str)
def run(app, arguments, version):
    """Runs the given application."""
    _run(app, arguments, version)


def _run(app, arguments, version=None):
    tool = find_app(app, version)
    if not tool:
        raise OzyError(f"Unable to find ozy-controlled app '{app}'")
    tool.ensure_installed()
    try:
        # The child process shouldn't get any of our overridden variables; put the original ones back.
        environment = restore_overridden_env_vars(os.environ)
        os.execve(tool.executable, [tool.executable] + list(arguments), environment)
    except Exception as e:
        _LOGGER.error("Unable to execute %s: %s", tool.executable, e)
        raise


def app_main(path_to_ozy, argv0, arguments, is_single_file):
    global PATH_TO_ME, IS_SINGLE_FILE
    PATH_TO_ME = os.path.realpath(path_to_ozy)
    IS_SINGLE_FILE = is_single_file

    invoked_as = os.path.basename(argv0)
    if invoked_as in ('ozy', '__main__.py'):
        main(prog_name='ozy', args=arguments)
    else:
        coloredlogs.install(fmt='%(message)s', level='INFO')
        _run(invoked_as, arguments)


if __name__ == "__main__":
    try:
        app_main(sys.argv[1], sys.argv[1], sys.argv[2:], False)
    except OzyError as ozy_error:
        _LOGGER.error(ozy_error)
        _LOGGER.debug(ozy_error, exc_info=True)
        sys.exit(1)
