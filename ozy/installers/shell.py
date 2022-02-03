import os
from subprocess import check_call
from tempfile import NamedTemporaryFile

from ozy.installer import Installer
from ozy.utils import download_to_file_obj


class ShellInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url', 'shell_args', extra_path_during_install='')

    def __str__(self):
        return f'shell file installer from {self.config("url")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        url = self.config('url')
        with NamedTemporaryFile(delete=False) as temp_file:
            download_to_file_obj(temp_file, url)
            temp_file.close()
            env = os.environ.copy()
            extra_path_during_install = self.config('extra_path_during_install')
            if extra_path_during_install:
                extra_path_during_install = os.path.join(to_dir, extra_path_during_install)
                env['PATH'] = f"{extra_path_during_install}:{env['PATH']}"
            env['INSTALL_DIR'] = to_dir
            check_call(["/bin/bash", temp_file.name] + self.config("shell_args"), env=env)
            os.unlink(temp_file.name)
