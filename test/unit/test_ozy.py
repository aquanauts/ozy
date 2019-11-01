import pytest

from ozy import safe_expand, OzyException


def test_safe_expand():
    sample_config = {
        "terraform": {
            "version": "0.12.10",
            "url": "https://releases.hashicorp.com/terraform/{version}/terraform_{version}_{hashicorp_os}_amd64.zip"
        },
    }

    tool_info = sample_config['terraform']
    expanded_config = safe_expand(dict(version="0.12.10", hashicorp_os="linux"), tool_info['url'])
    assert expanded_config == "https://releases.hashicorp.com/terraform/0.12.10/terraform_0.12.10_linux_amd64.zip"


def test_bad_safe_expand():
    with pytest.raises(OzyException):
        safe_expand(dict(foo="bar"), "I am templated {baz}")