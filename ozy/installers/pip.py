import logging
import os
from subprocess import check_call

from ozy.installer import Installer

_LOGGER = logging.getLogger(__name__)


class PipInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'package', 'version', channels=[])

    def __str__(self):
        return f'pip app installer for {self.config("package")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        channels = []
        for channel in self.config('channels'):
            channels += ['-c', channel]
        # Create a conda environment with pip installed (which brings in python et al)
        args = ['conda', 'create', '-y'] + channels + ['-p', to_dir, 'pip']
        _LOGGER.debug("Executing %s", " ".join(args))
        check_call(args)
        # Use that environment to pip install the user's package
        args = [os.path.join(to_dir, 'bin', 'pip'), 'install', f'{self.config("package")}=={self.config("version")}']
        _LOGGER.debug("Executing %s", " ".join(args))
        check_call(args)
