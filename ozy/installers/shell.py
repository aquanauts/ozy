import os
from subprocess import check_call
from tempfile import NamedTemporaryFile

from ozy.installer import Installer
from ozy.utils import download_to_file_obj


class ShellInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url', 'shell_args')

    def __str__(self):
        return f'shell file installer from {self.config("url")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        url = self.config('url')
        with NamedTemporaryFile(delete=False) as temp_file:
            download_to_file_obj(temp_file, url)
            temp_file.close()
            env = os.environ.copy()
            env['INSTALL_DIR'] = to_dir
            check_call(["/bin/bash", temp_file.name] + self.config("shell_args"), env=env)
            os.unlink(temp_file.name)
