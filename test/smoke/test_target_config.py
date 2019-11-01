import requests


def isValidInt(s):
    try:
        int(s)
        return True
    except ValueError:
        return False


def test_can_validate_a_hashicorp_installer():
    config = {
        "version-string": "0.9.4",
        "download-url": "https://releases.hashicorp.com/nomad/0.9.4/nomad_0.9.4_linux_amd64.zip",
        "download-sha256sum-url": "https://releases.hashicorp.com/nomad/0.9.4/nomad_0.9.4_SHA256SUMS"
    }

    {
        "terraform": {
            "version": "0.12.10",
            "url": "https://releases.hashicorp.com/terraform/{version}/terraform_{version}_{hashicorp_os}_amd64.zip"
        },
        "nomad": {
            "version": "0.9.5",
            "url": "https://releases.hashicorp.com/nomad/{version}/nomad_{version}_{hashicorp_os}_amd64.zip"
        },
        "vault": {
            "version": "1.1.3",
            "url": "https://releases.hashicorp.com/vault/{version}/vault_{version}_{hashicorp_os}_amd64.zip"
        }
    }

    response = requests.get(config['download-url'], stream=True)

    assert response.status_code == 200

    headers = response.headers
    content_length = headers['Content-length']
    assert isValidInt(content_length)
    content_length = int(content_length)
    assert content_length > 0
    print(content_length)
