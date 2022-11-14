import os

from textwrap import dedent

from ozy.installer import Installer
from ozy.utils import download_to_file_obj


class DockerInstaller(Installer):
    def __init__(self, name, config):
        super().__init__(name, config, 'docker_run_args', 'docker_image', app_name=name)

    def __str__(self):
        return f'docker installer from {self.config("cmd")}'

    def install(self, to_dir):
        os.makedirs(to_dir)
        docker_image = self.config('docker_image')
        docker_run_args = self.config('docker_run_args')
        app_path = os.path.join(to_dir, self.config('app_name'))
        cmd_file = dedent(
            f"""
            #!/bin/bash

            docker run --rm {docker_run_args} -it {docker_image} "$@"
            """
        )
        with open(app_path, 'wb') as output_file:
            output_file.write(cmd_file)
            
        os.chmod(app_path, 0o774)
