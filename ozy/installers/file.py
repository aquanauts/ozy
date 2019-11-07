import os

from ozy.installer import Installer
from ozy.utils import download_to_file_obj


class SingleFileInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url', app_name=name)

    def __str__(self):
        return f'file installer from {self.config("url")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        url = self.config('url')
        app_path = os.path.join(to_dir, self.config('app_name'))
        with open(app_path, 'wb') as output_file:
            download_to_file_obj(output_file, url)
        os.chmod(app_path, 0o774)
