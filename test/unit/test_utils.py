from ozy.utils import restore_overridden_env_vars


def test_restore_original_env_vars_keeps_existing():
    orig = dict(a='a', b='b', c='c')
    assert restore_overridden_env_vars(orig) == orig


def test_restore_original_env_vars_restores_pythonpath():
    orig = dict(PYTHONPATH='new', PYTHONPATH_ORIG='orig')
    assert restore_overridden_env_vars(orig) == dict(PYTHONPATH='orig')


def test_restore_original_env_vars_restores_ld_library_path():
    orig = dict(LD_LIBRARY_PATH='new', LD_LIBRARY_PATH_ORIG='orig')
    assert restore_overridden_env_vars(orig) == dict(LD_LIBRARY_PATH='orig')


def test_restore_original_env_vars_unsets_pythonpath_and_ld_path_if_not_originally_set():
    orig = dict(PYTHONPATH='new', LD_LIBRARY_PATH='new')
    assert restore_overridden_env_vars(orig) == dict()
