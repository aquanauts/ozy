import pytest

from ozy import OzyError
from ozy.installer import Installer


def test_empty_dict_is_ok():
    Installer('test', dict())


def test_drops_non_specified_keys():
    installer = Installer('test', dict(not_mentioned='123'))
    with pytest.raises(KeyError):
        installer.config('not_mentioned')


def test_accepts_non_required_keys():
    installer = Installer('test', dict(optional_key='opt'), optional_key='some default')
    assert installer.config('optional_key') == 'opt'


def test_defaults_non_required_keys():
    installer = Installer('test', dict(), optional_key='some default')
    assert installer.config('optional_key') == 'some default'


def test_accepts_required_keys():
    installer = Installer('test', dict(version='1.2.3'), 'version')
    assert installer.config('version') == '1.2.3'


def test_throws_on_missing_required_keys():
    with pytest.raises(OzyError, match='Missing required key.*not_there.*in.*test.*'):
        Installer('test', dict(), 'not_there')
