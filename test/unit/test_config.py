import os
from shutil import copyfile
from tempfile import TemporaryDirectory

import pytest
import yaml

from ozy import OzyError
from ozy.config import safe_expand, parse_ozy_conf, resolve, apply_overrides, get_user_conf_file, load_config


def test_parse_ozy_conf():
    sample_config_file = "conf/sample-team-conf.yaml"
    ozy_conf = parse_ozy_conf(sample_config_file)
    assert ozy_conf is not None
    assert 'name' in ozy_conf
    assert 'ozy_version' in ozy_conf
    assert 'templates' in ozy_conf
    assert 'apps' in ozy_conf

    templates = ozy_conf['templates']
    assert 'hashicorp' in templates


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


def test_resolve():
    sample_config_file = "conf/sample-team-conf.yaml"
    ozy_conf = parse_ozy_conf(sample_config_file)
    templates = ozy_conf['templates']
    config = ozy_conf['apps']['nomad']

    with pytest.raises(OzyError):
        bad_templates = templates.copy()
        del bad_templates['hashicorp']
        resolve(config, bad_templates)

    # try intentionally switching them
    with pytest.raises(OzyError):
        resolve(config=templates, templates=config)

    good_resolved = resolve(config, templates)
    assert 'url' in good_resolved
    assert 'template' in good_resolved
    assert 'type' in good_resolved
    assert 'app_name' in good_resolved
    assert 'version' in good_resolved
    assert 'sha256' in good_resolved
    assert 'sha256_gpg_key' in good_resolved


def test_apply_overrides():
    # test the flat case
    source = {'baz': 'super-rab'}
    dest = {'baz': 'rab'}
    result = apply_overrides(source, dest)
    assert result['baz'] == 'super-rab'

    # test the nested dict case
    source = {'foo': { 'bar': 'super-baz'} }
    dest = {'foo': { 'bar': 'baz'} }
    result = apply_overrides(source, dest)
    assert result['foo']['bar'] == 'super-baz'


def save_temp_ozy_conf(config, target_path):
    with open(target_path, "w") as fobj:
        yaml.dump(config, fobj)

def test_load_config():

    with TemporaryDirectory() as td:
        subdirA = os.path.join(td, "A")
        subdirA1 = os.path.join(subdirA, "1")

        os.mkdir(subdirA)
        os.mkdir(subdirA1)

        sample_config = 'conf/sample-team-conf.yaml'
        subdirA_config = parse_ozy_conf(sample_config)
        subdirA_config['apps']['nomad']['version'] = '10.10.10.10'
        save_temp_ozy_conf(subdirA_config, os.path.join(subdirA, ".ozy.yaml"))

        loaded_config = load_config(config=parse_ozy_conf(sample_config),
                                    current_working_directory=subdirA)

        assert loaded_config['apps']['nomad']['version'] == '10.10.10.10'

        #lets try with three dirs!
        subdirA1_config = parse_ozy_conf(sample_config)
        subdirA1_config['apps']['nomad']['version'] = '11.11.11.11'
        save_temp_ozy_conf(subdirA1_config, os.path.join(subdirA1, ".ozy.yaml"))

        loaded_config = load_config(config=parse_ozy_conf(sample_config),
                                    current_working_directory=subdirA1)

        assert loaded_config['apps']['nomad']['version'] == '11.11.11.11'


def test_get_user_conf_file():
    ucf = get_user_conf_file()
    assert ucf

