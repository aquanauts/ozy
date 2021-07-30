import logging
import os
from pathlib import Path
from subprocess import check_call
from tempfile import TemporaryDirectory
from typing import List

from ozy.installer import Installer

_LOGGER = logging.getLogger(__name__)


class CondaInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'package', 'version', channels=[], conda_bin="conda", pyinstaller=False)

    def __str__(self):
        return f'conda app installer for {self.config("package")}'

    def _conda_install(self, to_dir: str, to_install: List[str]):
        channels = []
        for channel in self.config('channels'):
            channels += ['-c', channel]
        conda_bin = self.config("conda_bin")
        args = [conda_bin, 'create', '-y'] + channels + ['-p', to_dir] + to_install
        _LOGGER.debug("Executing %s", " ".join(args))
        check_call(args)

    def install(self, to_dir):
        versioned_package = f'{self.config("package")}={self.config("version")}'
        if self.config('pyinstaller'):
            with TemporaryDirectory(prefix='ozy-conda-installer') as td:
                self._conda_install(td, [versioned_package, 'pyinstaller'])
                pyinst = Path(td) / 'bin' / 'pyinstaller'
                dest = (Path(to_dir) / self._executable_path).parent
                args = [
                    str(pyinst),
                    '--onefile',
                    '--name', self._name,
                    '--distpath', str(dest),
                    str(Path(td) / self._executable_path)
                ]
                _LOGGER.debug("Executing %s", " ".join(args))
                check_call(args)
        else:
            os.makedirs(to_dir)
            self._conda_install(to_dir, [versioned_package])
