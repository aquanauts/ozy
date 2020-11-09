from ozy.installers.conda import CondaInstaller
from ozy.installers.file import SingleFileInstaller
from ozy.installers.pip import PipInstaller
from ozy.installers.shell import ShellInstaller
from ozy.installers.tar import TarballInstaller
from ozy.installers.zip import SingleBinaryZipInstaller, ZipInstaller

SUPPORTED_INSTALLERS = dict(
    single_binary_zip=SingleBinaryZipInstaller,
    zip=ZipInstaller,
    tarball=TarballInstaller,
    single_file=SingleFileInstaller,
    shell_install=ShellInstaller,
    conda=CondaInstaller,
    pip=PipInstaller
)
