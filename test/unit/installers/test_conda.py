from unittest import mock

from ozy.installers import CondaInstaller


@mock.patch('ozy.installers.conda.os.makedirs')
@mock.patch('ozy.installers.conda.check_call')
def test_should_install_with_regular_installer(mock_check_call, mock_makedirs):
    installer = CondaInstaller('test', dict(package='package', version='1.0.0', channels=['chan1', 'chan2']))
    installer.install('/some/directory')
    mock_makedirs.assert_called_with('/some/directory')
    mock_check_call.assert_called_with([
        'conda', 'create', '-y',
        '-c', 'chan1',
        '-c', 'chan2',
        '-p', '/some/directory',
        'package=1.0.0'
    ])


@mock.patch('ozy.installers.conda.check_call')
def test_should_install_with_pyinstaller_squish(mock_check_call):
    installer = CondaInstaller('test',
                               dict(package='package', version='1.0.0', pyinstaller=True, channels=['chan1', 'chan2']))
    installer.install('/some/directory')
    mock_check_call.assert_has_calls([
        mock.call([
            'conda', 'create', '-y',
            '-c', 'chan1',
            '-c', 'chan2',
            '-p', mock.ANY,
            'package=1.0.0',
            'pyinstaller'
        ]),
        mock.call([
            mock.ANY,  # the unknown temporary file reference to pyinstaller
            '--onefile', '--name', 'test', '--distpath', '/some/directory',
            mock.ANY  # the unknown temporary file reference to the test installation
        ])
    ])
