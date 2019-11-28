from unittest import mock

import pytest

from ozy import OzyError
from ozy.installers.pip import PipInstaller


def test_raises_on_missing_keys():
    with pytest.raises(OzyError):
        PipInstaller('test', dict())
    with pytest.raises(OzyError):
        PipInstaller('test', dict(package='p'))
    with pytest.raises(OzyError):
        PipInstaller('test', dict(version='123'))


def test_constructs_ok_with_correct_config():
    PipInstaller('test', dict(package='p', version='123'))


@mock.patch('ozy.installers.pip.os.makedirs')
@mock.patch('ozy.installers.pip.check_call')
def test_installs(mock_check_call, mock_makedirs):
    installer = PipInstaller('test', dict(package='package', version='1.0.0'))
    installer.install('/some/directory')
    mock_makedirs.assert_called_with('/some/directory')
    mock_check_call.assert_has_calls([
        mock.call(['conda', 'create', '-y', '-p', '/some/directory', 'pip']),
        mock.call(['/some/directory/bin/pip', 'install', 'package==1.0.0'])
    ])
