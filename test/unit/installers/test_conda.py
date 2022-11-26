import os

from tempfile import TemporaryDirectory
from unittest import mock

from ozy.installers import CondaInstaller


@mock.patch('ozy.installers.conda.check_call')
def test_should_install_with_regular_installer(mock_check_call):
    with TemporaryDirectory() as root:
        installer = CondaInstaller('test', dict(package='package', version='1.0.0', channels=['chan1', 'chan2']))
        os.makedirs(root + '/some/directory')
        installer.install(root + '/some/directory')
        assert os.path.isdir(root + '/some/directory')
        mock_check_call.assert_called_with([
            'conda', 'create', '-y',
            '-c', 'chan1',
            '-c', 'chan2',
            '-p', root + '/some/directory',
            'package=1.0.0'
        ], env=mock.ANY)

@mock.patch('ozy.installers.conda.do_conda_install')
@mock.patch('ozy.installers.conda.check_call')
def test_should_install_with_pyinstaller_squish(mock_check_call, mock_do_conda_install):
    installer = CondaInstaller('test',
                               dict(package='package', version='1.0.0', pyinstaller=True, channels=['chan1', 'chan2']))
    installer.install('/some/directory')
    mock_do_conda_install.assert_called_with('conda', ['chan1', 'chan2'], mock.ANY, ['package=1.0.0', 'pyinstaller'])
    mock_check_call.assert_called_with([
        mock.ANY,  # the unknown temporary file reference to pyinstaller
        '--onefile', '--name', 'test', '--distpath', '/some/directory',
        mock.ANY  # the unknown temporary file reference to the test installation
    ])
