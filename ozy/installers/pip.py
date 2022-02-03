import logging
import os
from subprocess import check_call

from ozy.installer import Installer
from ozy.installers.conda import do_conda_install

_LOGGER = logging.getLogger(__name__)


class PipInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'package', 'version', channels=[])

    def __str__(self):
        return f'pip app installer for {self.config("package")}'

    def install(self, to_dir):
        # Create a conda environment with pip installed (which brings in python et al)
        do_conda_install('conda', self.config('channels'), to_dir, ['pip'])
        # Use that environment to pip install the user's package
        args = [os.path.join(to_dir, 'bin', 'pip'), 'install', f'{self.config("package")}=={self.config("version")}']
        _LOGGER.debug("Executing %s", " ".join(args))
        check_call(args)
