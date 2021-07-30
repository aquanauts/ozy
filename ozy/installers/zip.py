import os
from tempfile import NamedTemporaryFile
from zipfile import ZipFile

from ozy import OzyError
# TODO support sha256, sha256_signature and sha256_gpg_key
from ozy.installer import Installer
from ozy.utils import download_to_file_obj


class SingleBinaryZipInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url', app_name=name)

    def __str__(self):
        return f'single_binary_zip installer from {self.config("url")}'

    def install(self, to_dir):
        app_name = self.config('app_name')
        url = self.config('url')
        os.makedirs(to_dir)
        app_path = os.path.join(to_dir, app_name)
        with NamedTemporaryFile() as temp_file:
            download_to_file_obj(temp_file, url)
            temp_file.flush()
            zf = ZipFile(temp_file.name)
            contents = zf.namelist()
            if len(contents) != 1:
                raise OzyError(f"More than one file in the zipfile at {url}! ({contents})")
            with open(app_path, 'wb') as out_file:
                with zf.open(contents[0]) as in_file:
                    out_file.write(in_file.read())
            os.chmod(app_path, 0o774)


class ZipInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'url')

    def __str__(self):
        return f'zip installer from {self.config("url")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        url = self.config('url')
        with NamedTemporaryFile() as temp_file:
            download_to_file_obj(temp_file, url)
            temp_file.flush()
            zf = ZipFile(temp_file.name)
            zf.extractall(to_dir)

        os.chmod(os.path.join(to_dir, self._executable_path), 0o755)
