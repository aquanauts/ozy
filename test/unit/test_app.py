import pytest

from ozy import OzyError
from ozy.app import App, ensure_keys
from ozy.config import parse_ozy_conf, resolve


def test_app():
    sample_config_file = "conf/sample-team-conf.yaml"
    ozy_conf = parse_ozy_conf(sample_config_file)
    app = App("nomad", ozy_conf )
    assert app.name == "nomad"
    assert app.config['template'] == 'hashicorp'
    assert app.config['url']
    assert app.config['type'] == 'single_binary_zip'
    assert app.config['sha256_gpg_key'] == ozy_conf['templates']['hashicorp']['sha256_gpg_key']
    assert app.config['sha256']
    assert app.config['sha256_signature']

def test_unsupported_installer():
    sample_config_file = "conf/sample-team-conf.yaml"
    ozy_conf = parse_ozy_conf(sample_config_file)
    ozy_conf['templates']['hashicorp']['type'] = 'triple_binary_star' # not a supported installer
    with pytest.raises(OzyError):
        App("nomad", ozy_conf )


def test_ensure_keys():
    sample_config_file = "conf/sample-team-conf.yaml"
    ozy_conf = parse_ozy_conf(sample_config_file)

    tmp_ozy_conf = ozy_conf.copy()
    del tmp_ozy_conf['templates']['hashicorp']['type']
    config = resolve(ozy_conf['apps']['nomad'], ozy_conf['templates'])
    assert config
    with pytest.raises(OzyError):
        ensure_keys('nomad', config, 'type')


