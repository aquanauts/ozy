import os

import pytest

from ozy import safe_expand, OzyError, get_ozy_dir, walk_up_dirs


def test_safe_expand():
    sample_config = {
        "terraform": {
            "version": "0.12.10",
            "url": "https://releases.hashicorp.com/terraform/{version}/terraform_{version}_{hashicorp_os}_amd64.zip",
            "list": ['foo', '{version}', 'bar']
        },
    }

    tool_info = sample_config['terraform']
    expanded_config = safe_expand(dict(version="0.12.10", hashicorp_os="linux"), tool_info['url'])
    assert expanded_config == "https://releases.hashicorp.com/terraform/0.12.10/terraform_0.12.10_linux_amd64.zip"
    expanded_config = safe_expand(dict(version="0.12.10", hashicorp_os="linux"), tool_info['list'])
    assert expanded_config == ['foo', '0.12.10', 'bar']


def test_bad_safe_expand():
    with pytest.raises(OzyError):
        safe_expand(dict(foo="bar"), "I am templated {baz}")


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
