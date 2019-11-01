import logging
import os
import sys

import click

from ozy import OzyException, find_tool, install_if_needed_and_get_path_to_tool


@click.group()
def main():
    logging.basicConfig(level=logging.INFO)
    print("moo")


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
