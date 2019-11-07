import os
import tarfile
from tempfile import NamedTemporaryFile

from ozy.installer import Installer
from ozy.utils import download_to_file_obj


class TarballInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url')

    def __str__(self):
        return f'tar installer from {self.config("url")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        url = self.config('url')
        with NamedTemporaryFile() as temp_file:
            download_to_file_obj(temp_file, url)
            temp_file.flush()
            tf = tarfile.open(temp_file.name, 'r')
            tf.extractall(to_dir)
