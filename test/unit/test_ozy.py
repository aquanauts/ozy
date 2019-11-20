import os
import pytest
from ozy import OzyError
from ozy.files import walk_up_dirs, get_ozy_dir


def test_ozy_dirs():
    ozy_dir = get_ozy_dir()
    assert ozy_dir is not None
    home = os.environ['HOME']
    del os.environ['HOME']
    with pytest.raises(OzyError):
        get_ozy_dir()
    os.environ['HOME'] = home


def test_walk_up_dirs():
    test_path = os.path.join(os.path.sep, 'one', 'two', 'three')
    assert [
               os.path.join(os.path.sep, 'one', 'two', 'three'),
               os.path.join(os.path.sep, 'one', 'two'),
               os.path.join(os.path.sep, 'one'),
               os.path.sep
           ] == [x for x in walk_up_dirs(test_path)]






