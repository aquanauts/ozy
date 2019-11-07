import logging
import os
from typing import BinaryIO

import requests
from tqdm import tqdm

from ozy import OzyError

_LOGGER = logging.getLogger(__name__)
_DOWNLOAD_CHUNK_SIZE = 128 * 1024


def download_to_file_obj(dest_file_obj: BinaryIO, url: str):
    response = requests.get(url, stream=True)
    if not response.ok:
        raise OzyError(f"Unable to fetch url '{url}' - {response}")
    total_size = int(response.headers.get('content-length', 0))
    with tqdm(total=total_size, unit='iB', unit_scale=True) as t:
        for data in response.iter_content(_DOWNLOAD_CHUNK_SIZE):
            t.update(len(data))
            dest_file_obj.write(data)


def download_to(dest_file_name: str, url: str):
    _LOGGER.debug("Downloading %s to %s", url, dest_file_name)
    dest_file_temp = dest_file_name + ".tmp"
    try:
        with open(dest_file_temp, 'wb') as dest_file_obj:
            download_to_file_obj(dest_file_obj, url)
        os.rename(dest_file_temp, dest_file_name)
    except Exception:
        os.unlink(dest_file_temp)
        raise
