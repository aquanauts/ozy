import requests


def is_valid_int(s):
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

    response = requests.get(config['download-url'], stream=True)

    assert response.status_code == 200

    headers = response.headers
    content_length = headers['Content-length']
    assert is_valid_int(content_length)
    content_length = int(content_length)
    assert content_length > 0
    print(content_length)
