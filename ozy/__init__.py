
class OzyException(Exception):
    pass


def safe_expand(format_params, to_expand):
    try:
        return to_expand.format(**format_params)
    except KeyError as ke:
        raise OzyException(f"Could not find key {ke} in expansion '{to_expand}' with params '{format_params}'")
