import coloredlogs, logging
import os
import sys

import click

from ozy import OzyException, find_tool, install_if_needed_and_get_path_to_tool, download_to, \
    get_ozy_dir, ensure_ozy_dirs, get_ozy_bin_dir

_LOGGER = logging.getLogger(__name__)


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
    # TODO: make sure it isn't there already? upgrade/update instead?
    ozy_conf_filename = f"{get_ozy_dir()}/ozy.conf.yaml"
    download_to(ozy_conf_filename, url)
    ozy_bin_dir = os.path.realpath(get_ozy_bin_dir())
    real_paths = set(os.path.realpath(path) for path in os.getenv("PATH").split(":"))
    if ozy_bin_dir not in real_paths:
        _LOGGER.warning("Please place '%s' on your path", ozy_bin_dir)
        # TODO make this nicer! instructions, helpers, etc
    # Ensure on path!
    # Copy ourselves there? <tricky>
    # for app in parse_ozy_conf(ozy_conf_filename):
    #     do the symlink thing!
    print(f"WE R TEH INSTALLXOR {url}")


if __name__ == "__main__":
    invoked_as = os.path.basename(sys.argv[1])
    sys.argv = [sys.argv[0]] + sys.argv[2:]
    if invoked_as in ('ozy', '__main__.py'):
        main()
    else:
        tool = find_tool(invoked_as)
        if not tool:
            raise OzyException(f"TODO better, couldn't find {invoked_as}")
        path = install_if_needed_and_get_path_to_tool(tool)
        os.execv(path, [path] + sys.argv[1:])
